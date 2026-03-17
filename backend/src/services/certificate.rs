use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use instant_acme::{
    Account, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder,
    OrderStatus,
};
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, EntityTrait,
    ModelTrait, QueryFilter, Set,
};
use x509_parser::extensions::GeneralName;

use crate::config::Config;
use crate::db::entities::{
    certificate::{self, ActiveModel, CertProvider, Entity as CertEntity},
    website::{self, Entity as WebsiteEntity, ServerType},
};
use crate::error::{AppError, AppResult};
use crate::services::application::ApplicationManager;
use crate::services::docker::DockerService;
use crate::services::proxy_config::ProxyConfigService;

const CERTS_BASE_DIR: &str = "/opt/mana-panel/certs";
const ACME_ACCOUNT_PATH: &str = "/opt/mana-panel/certs/acme-account.json";
pub const ACME_CHALLENGE_DIR: &str = "/opt/mana-panel/acme-challenge";

/// Manages SSL certificate lifecycle: issuance via ACME (Let's Encrypt) and custom upload.
pub struct CertificateService;

impl CertificateService {
    /// Issue a certificate from Let's Encrypt using HTTP-01 challenge.
    ///
    /// The caller must ensure that the ACME challenge directory is being served
    /// by the web server at `/.well-known/acme-challenge/` before calling this.
    pub async fn issue_certificate(
        db: &DatabaseConnection,
        domain: &str,
        aliases: &[String],
        email: Option<&str>,
    ) -> AppResult<certificate::Model> {
        let domain = Self::normalize_and_validate_domain(domain)?;
        let all_domains = Self::normalize_and_validate_domains(&domain, aliases)?;

        let challenge_webroot = PathBuf::from(ACME_CHALLENGE_DIR);
        let challenge_dir = challenge_webroot.join(".well-known").join("acme-challenge");
        std::fs::create_dir_all(&challenge_dir).map_err(|e| {
            AppError::System(format!("Failed to create ACME challenge directory: {}", e))
        })?;

        // Build list of identifiers (primary domain + aliases)
        let identifiers = all_domains
            .iter()
            .map(|d| Identifier::Dns(d.clone()))
            .collect::<Vec<_>>();

        // Load or create ACME account
        let account = Self::load_or_create_account(email).await?;

        // Create order
        let mut order = account
            .new_order(&NewOrder {
                identifiers: &identifiers,
            })
            .await
            .map_err(|e| AppError::System(format!("Failed to create ACME order: {}", e)))?;

        let authorizations = order
            .authorizations()
            .await
            .map_err(|e| AppError::System(format!("Failed to get ACME authorizations: {}", e)))?;

        // Process each authorization's HTTP-01 challenge
        let mut challenge_tokens = Vec::new();
        for auth in &authorizations {
            match auth.status {
                AuthorizationStatus::Valid => continue,
                AuthorizationStatus::Pending => {}
                status => {
                    return Err(AppError::System(format!(
                        "Unexpected authorization status: {:?}",
                        status
                    )));
                }
            }

            let challenge = auth
                .challenges
                .iter()
                .find(|c| c.r#type == ChallengeType::Http01)
                .ok_or_else(|| {
                    AppError::System("No HTTP-01 challenge found for authorization".to_string())
                })?;

            let token = &challenge.token;
            let key_authorization = order.key_authorization(challenge);

            // Write challenge response file
            let challenge_file = challenge_dir.join(token);
            std::fs::write(&challenge_file, key_authorization.as_str()).map_err(|e| {
                AppError::System(format!("Failed to write ACME challenge file: {}", e))
            })?;

            challenge_tokens.push((challenge_file, challenge.url.clone()));
            tracing::info!(
                "Wrote ACME HTTP-01 challenge for token '{}' at {}",
                token,
                challenge_dir.display()
            );
        }

        // Signal all challenges ready
        for (_, challenge_url) in &challenge_tokens {
            order
                .set_challenge_ready(challenge_url)
                .await
                .map_err(|e| {
                    Self::cleanup_challenge_files(&challenge_tokens);
                    AppError::System(format!("Failed to signal challenge ready: {}", e))
                })?;
        }

        // Poll for order to become ready (up to 30 attempts, 2s apart)
        let mut attempts = 0;
        let state = loop {
            attempts += 1;
            if attempts > 30 {
                Self::cleanup_challenge_files(&challenge_tokens);
                return Err(AppError::System(
                    "Timed out waiting for ACME order to become ready".to_string(),
                ));
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let state = order.refresh().await.map_err(|e| {
                Self::cleanup_challenge_files(&challenge_tokens);
                AppError::System(format!("Failed to refresh ACME order: {}", e))
            })?;

            match state.status {
                OrderStatus::Ready => break state,
                OrderStatus::Invalid => {
                    Self::cleanup_challenge_files(&challenge_tokens);
                    return Err(AppError::System(
                        "ACME order became invalid. Ensure the domain points to this server and port 80 is accessible."
                            .to_string(),
                    ));
                }
                OrderStatus::Valid => break state,
                OrderStatus::Pending | OrderStatus::Processing => continue,
            }
        };

        // Clean up challenge files now that validation is done
        Self::cleanup_challenge_files(&challenge_tokens);

        // Generate private key and CSR
        let private_key = KeyPair::generate()
            .map_err(|e| AppError::System(format!("Failed to generate private key: {}", e)))?;

        let mut csr_params = CertificateParams::new(
            identifiers
                .iter()
                .map(|id| match id {
                    Identifier::Dns(d) => d.clone(),
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| AppError::System(format!("Failed to create CSR params: {}", e)))?;
        csr_params.distinguished_name = DistinguishedName::new();

        let csr = csr_params
            .serialize_request(&private_key)
            .map_err(|e| AppError::System(format!("Failed to serialize CSR: {}", e)))?;

        let csr_der = csr.der();

        // Finalize order with CSR (only if not already valid)
        if state.status != OrderStatus::Valid {
            order
                .finalize(csr_der)
                .await
                .map_err(|e| AppError::System(format!("Failed to finalize ACME order: {}", e)))?;

            // Poll for certificate
            let mut cert_attempts = 0;
            loop {
                cert_attempts += 1;
                if cert_attempts > 15 {
                    return Err(AppError::System(
                        "Timed out waiting for certificate issuance".to_string(),
                    ));
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                let state = order
                    .refresh()
                    .await
                    .map_err(|e| AppError::System(format!("Failed to refresh order: {}", e)))?;
                if state.status == OrderStatus::Valid {
                    break;
                }
                if state.status == OrderStatus::Invalid {
                    return Err(AppError::System(
                        "ACME order became invalid during finalization".to_string(),
                    ));
                }
            }
        }

        // Download certificate chain
        let cert_chain_pem = order
            .certificate()
            .await
            .map_err(|e| AppError::System(format!("Failed to download certificate: {}", e)))?
            .ok_or_else(|| AppError::System("No certificate returned by ACME".to_string()))?;

        let key_pem = private_key.serialize_pem();

        // Write cert and key to disk
        let (cert_path, key_path) = Self::write_cert_files(&domain, &cert_chain_pem, &key_pem)?;

        // Parse expiry from certificate
        let expires_at = Self::parse_cert_expiry(&cert_chain_pem)?;

        // Upsert in database
        let model = Self::upsert_certificate(
            db,
            &domain,
            CertProvider::LetsEncrypt,
            &cert_path,
            &key_path,
            expires_at,
        )
        .await?;

        tracing::info!(
            "Successfully issued Let's Encrypt certificate for '{}', expires {}",
            domain,
            expires_at
        );

        Ok(model)
    }

    /// Upload a custom certificate (PEM-encoded cert chain + private key).
    pub async fn upload_certificate(
        db: &DatabaseConnection,
        domain: &str,
        cert_pem: &str,
        key_pem: &str,
    ) -> AppResult<certificate::Model> {
        let domain = Self::normalize_and_validate_domain(domain)?;
        Self::validate_uploaded_certificate(&domain, cert_pem, key_pem)?;

        let expires_at = Self::parse_cert_expiry(cert_pem)?;

        let (cert_path, key_path) = Self::write_cert_files(&domain, cert_pem, key_pem)?;

        let model = Self::upsert_certificate(
            db,
            &domain,
            CertProvider::Custom,
            &cert_path,
            &key_path,
            expires_at,
        )
        .await?;

        tracing::info!("Uploaded custom certificate for '{}'", domain);

        Ok(model)
    }

    /// Get certificate for a domain.
    pub async fn get_by_domain(
        db: &DatabaseConnection,
        domain: &str,
    ) -> AppResult<Option<certificate::Model>> {
        let domain = domain.trim().trim_end_matches('.').to_ascii_lowercase();
        let result = CertEntity::find()
            .filter(certificate::Column::Domain.eq(domain))
            .one(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to query certificate: {}", e)))?;
        Ok(result)
    }

    /// List all certificates.
    pub async fn list(db: &DatabaseConnection) -> AppResult<Vec<certificate::Model>> {
        let results = CertEntity::find()
            .all(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to list certificates: {}", e)))?;
        Ok(results)
    }

    /// Delete a certificate by ID.
    pub async fn delete(db: &DatabaseConnection, id: i32) -> AppResult<()> {
        let cert = CertEntity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to find certificate: {}", e)))?
            .ok_or_else(|| AppError::NotFound(format!("Certificate with id {} not found", id)))?;

        let cert_domain_normalized = cert.domain.trim().to_ascii_lowercase();
        let active_https_sites = WebsiteEntity::find()
            .filter(website::Column::HasSsl.eq(true))
            .filter(
                Condition::any()
                    .add(website::Column::ServerType.eq(ServerType::Nginx))
                    .add(website::Column::ServerType.eq(ServerType::OpenResty)),
            )
            .all(db)
            .await
            .map_err(|e| {
                AppError::System(format!(
                    "Failed to check certificate usage before delete: {}",
                    e
                ))
            })?;

        let in_use_sites: Vec<_> = active_https_sites
            .into_iter()
            .filter(|site| {
                site.primary_domain
                    .trim()
                    .to_ascii_lowercase()
                    .eq(&cert_domain_normalized)
            })
            .collect();

        if !in_use_sites.is_empty() {
            let site_names = in_use_sites
                .iter()
                .map(|site| site.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(AppError::Validation(format!(
                "Certificate for domain '{}' is in use by HTTPS website(s): {}. Disable HTTPS or change domain before deleting the certificate.",
                cert.domain, site_names
            )));
        }

        // Remove files
        let _ = std::fs::remove_file(&cert.cert_path);
        let _ = std::fs::remove_file(&cert.key_path);

        // Try to remove the domain directory if empty
        if let Some(parent) = Path::new(&cert.cert_path).parent() {
            let _ = std::fs::remove_dir(parent);
        }

        let domain = cert.domain.clone();
        cert.delete(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to delete certificate: {}", e)))?;

        tracing::info!("Deleted certificate {} for domain '{}'", id, domain);
        Ok(())
    }

    /// Returns the expected cert/key file paths for a domain.
    pub fn cert_paths(domain: &str) -> (String, String) {
        let dir = PathBuf::from(CERTS_BASE_DIR)
            .join(domain.trim().trim_end_matches('.').to_ascii_lowercase());
        (
            dir.join("fullchain.pem").to_string_lossy().to_string(),
            dir.join("privkey.pem").to_string_lossy().to_string(),
        )
    }

    /// Returns true when the certificate file referenced by this DB model covers
    /// the primary domain and all provided aliases.
    pub fn stored_certificate_covers_domains(
        cert: &certificate::Model,
        primary_domain: &str,
        aliases: &[String],
    ) -> AppResult<bool> {
        let required_domains = Self::normalize_and_validate_domains(primary_domain, aliases)?;

        let cert_pem = match std::fs::read_to_string(&cert.cert_path) {
            Ok(content) => content,
            Err(e) => {
                tracing::warn!(
                    "Failed to read existing certificate file '{}': {}. Will treat as not covering requested domains.",
                    cert.cert_path,
                    e
                );
                return Ok(false);
            }
        };

        Self::certificate_pem_covers_domains(&cert_pem, &required_domains)
    }

    // ── Internals ──

    fn write_cert_files(
        domain: &str,
        cert_pem: &str,
        key_pem: &str,
    ) -> AppResult<(String, String)> {
        let cert_dir = PathBuf::from(CERTS_BASE_DIR).join(domain);
        std::fs::create_dir_all(&cert_dir).map_err(|e| {
            AppError::System(format!(
                "Failed to create certificate directory '{}': {}",
                cert_dir.display(),
                e
            ))
        })?;

        let cert_path = cert_dir.join("fullchain.pem");
        let key_path = cert_dir.join("privkey.pem");

        std::fs::write(&cert_path, cert_pem)
            .map_err(|e| AppError::System(format!("Failed to write certificate file: {}", e)))?;
        std::fs::write(&key_path, key_pem)
            .map_err(|e| AppError::System(format!("Failed to write private key file: {}", e)))?;

        // Restrict key file permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
        }

        Ok((
            cert_path.to_string_lossy().to_string(),
            key_path.to_string_lossy().to_string(),
        ))
    }

    fn validate_uploaded_certificate(domain: &str, cert_pem: &str, key_pem: &str) -> AppResult<()> {
        let cert_pems = pem::parse_many(cert_pem)
            .map_err(|e| AppError::Validation(format!("Invalid certificate PEM: {}", e)))?;
        let cert_der = cert_pems
            .first()
            .ok_or_else(|| AppError::Validation("No certificate found in PEM".to_string()))?;
        let (_, cert) = x509_parser::parse_x509_certificate(cert_der.contents())
            .map_err(|e| AppError::Validation(format!("Invalid X.509 certificate data: {}", e)))?;

        if !Self::certificate_covers_domain(&cert, domain) {
            return Err(AppError::Validation(format!(
                "Uploaded certificate does not cover domain '{}'",
                domain.trim()
            )));
        }

        let key_pair = KeyPair::from_pem(key_pem).map_err(|e| {
            AppError::Validation(format!(
                "Invalid private key PEM or unsupported key format: {}",
                e
            ))
        })?;
        let key_public_spki = key_pair.public_key_der();
        let cert_public_spki = cert.tbs_certificate.subject_pki.raw;

        if cert_public_spki != key_public_spki.as_slice() {
            return Err(AppError::Validation(
                "Certificate and private key do not match".to_string(),
            ));
        }

        Ok(())
    }

    fn certificate_pem_covers_domains(cert_pem: &str, domains: &[String]) -> AppResult<bool> {
        let cert_pems = pem::parse_many(cert_pem)
            .map_err(|e| AppError::System(format!("Invalid certificate PEM: {}", e)))?;
        let cert_der = cert_pems
            .first()
            .ok_or_else(|| AppError::System("No certificate found in PEM".to_string()))?;
        let (_, cert) = x509_parser::parse_x509_certificate(cert_der.contents())
            .map_err(|e| AppError::System(format!("Invalid X.509 certificate data: {}", e)))?;

        Ok(domains
            .iter()
            .all(|domain| Self::certificate_covers_domain(&cert, domain)))
    }

    fn normalize_and_validate_domains(primary: &str, aliases: &[String]) -> AppResult<Vec<String>> {
        let mut domains = BTreeSet::new();
        domains.insert(Self::normalize_and_validate_domain(primary)?);

        for alias in aliases {
            let trimmed = alias.trim();
            if trimmed.is_empty() {
                continue;
            }
            domains.insert(Self::normalize_and_validate_domain(trimmed)?);
        }

        Ok(domains.into_iter().collect())
    }

    fn normalize_and_validate_domain(domain: &str) -> AppResult<String> {
        let normalized = domain.trim().trim_end_matches('.').to_ascii_lowercase();
        if normalized.is_empty() {
            return Err(AppError::Validation("Domain cannot be empty".to_string()));
        }

        if normalized.contains('/') || normalized.contains('\\') || normalized.contains("..") {
            return Err(AppError::Validation(
                "Domain contains invalid characters".to_string(),
            ));
        }

        if !Self::is_valid_hostname(&normalized) {
            return Err(AppError::Validation(format!(
                "'{}' is not a valid domain name",
                domain.trim()
            )));
        }

        Ok(normalized)
    }

    fn is_valid_hostname(host: &str) -> bool {
        if host.is_empty() || host.len() > 253 {
            return false;
        }

        for label in host.split('.') {
            if label.is_empty() || label.len() > 63 {
                return false;
            }
            if label.starts_with('-') || label.ends_with('-') {
                return false;
            }
            if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return false;
            }
        }

        true
    }

    fn certificate_covers_domain(
        cert: &x509_parser::certificate::X509Certificate<'_>,
        domain: &str,
    ) -> bool {
        match cert.subject_alternative_name() {
            Ok(Some(san)) => {
                let dns_names = san
                    .value
                    .general_names
                    .iter()
                    .filter_map(|name| match name {
                        GeneralName::DNSName(dns) => Some(*dns),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if !dns_names.is_empty() {
                    return dns_names
                        .into_iter()
                        .any(|dns| Self::dns_name_matches_pattern(domain, dns));
                }
            }
            Ok(None) => {}
            Err(_) => return false,
        }

        cert.subject()
            .iter_common_name()
            .filter_map(|cn| cn.as_str().ok())
            .any(|cn| Self::dns_name_matches_pattern(domain, cn))
    }

    fn dns_name_matches_pattern(domain: &str, pattern: &str) -> bool {
        let domain = domain.trim().trim_end_matches('.').to_ascii_lowercase();
        let pattern = pattern.trim().trim_end_matches('.').to_ascii_lowercase();

        if domain.is_empty() || pattern.is_empty() {
            return false;
        }

        if domain == pattern {
            return true;
        }

        if let Some(suffix) = pattern.strip_prefix("*.") {
            if !domain.ends_with(suffix) {
                return false;
            }
            let prefix = &domain[..domain.len() - suffix.len()];
            if !prefix.ends_with('.') {
                return false;
            }
            let wildcard_label = &prefix[..prefix.len() - 1];
            return !wildcard_label.is_empty() && !wildcard_label.contains('.');
        }

        false
    }

    fn parse_cert_expiry(cert_pem: &str) -> AppResult<chrono::DateTime<Utc>> {
        let pem_items = pem::parse_many(cert_pem)
            .map_err(|e| AppError::System(format!("Invalid PEM: {}", e)))?;

        let cert_der = pem_items
            .first()
            .ok_or_else(|| AppError::System("No certificate found in PEM".to_string()))?;

        let (_, cert) = x509_parser::parse_x509_certificate(cert_der.contents())
            .map_err(|e| AppError::System(format!("Failed to parse X.509 certificate: {}", e)))?;

        let not_after = cert.validity().not_after;
        let timestamp = not_after.timestamp();
        let expires = chrono::DateTime::<Utc>::from_timestamp(timestamp, 0)
            .ok_or_else(|| AppError::System("Invalid certificate expiry timestamp".to_string()))?;
        Ok(expires)
    }

    async fn upsert_certificate<C: ConnectionTrait>(
        db: &C,
        domain: &str,
        provider: CertProvider,
        cert_path: &str,
        key_path: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> AppResult<certificate::Model> {
        let now = Utc::now();

        // Check if a certificate already exists for this domain
        let existing = CertEntity::find()
            .filter(certificate::Column::Domain.eq(domain))
            .one(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to query certificate: {}", e)))?;

        if let Some(existing) = existing {
            let mut am: ActiveModel = existing.into();
            am.provider = Set(provider);
            am.cert_path = Set(cert_path.to_string());
            am.key_path = Set(key_path.to_string());
            am.expires_at = Set(expires_at);
            am.updated_at = Set(now);
            let updated = am
                .update(db)
                .await
                .map_err(|e| AppError::System(format!("Failed to update certificate: {}", e)))?;
            Ok(updated)
        } else {
            let am = ActiveModel {
                domain: Set(domain.to_string()),
                provider: Set(provider),
                cert_path: Set(cert_path.to_string()),
                key_path: Set(key_path.to_string()),
                expires_at: Set(expires_at),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };
            let inserted = am
                .insert(db)
                .await
                .map_err(|e| AppError::System(format!("Failed to insert certificate: {}", e)))?;
            Ok(inserted)
        }
    }

    async fn load_or_create_account(email: Option<&str>) -> AppResult<Account> {
        let account_path = Path::new(ACME_ACCOUNT_PATH);

        // Try to load existing account
        if account_path.exists() {
            if let Ok(json) = std::fs::read_to_string(account_path) {
                if let Ok(credentials) =
                    serde_json::from_str::<instant_acme::AccountCredentials>(&json)
                {
                    let account = Account::from_credentials(credentials).await.map_err(|e| {
                        AppError::System(format!(
                            "Failed to restore ACME account from credentials: {}",
                            e
                        ))
                    })?;
                    tracing::debug!(
                        "Loaded existing ACME account from {}",
                        account_path.display()
                    );
                    return Ok(account);
                }
            }
        }

        // Create new account
        let mut contact = Vec::new();
        if let Some(email) = email {
            if !email.trim().is_empty() {
                contact.push(format!("mailto:{}", email.trim()));
            }
        }

        let (account, credentials) = Account::create(
            &NewAccount {
                contact: &contact.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                terms_of_service_agreed: true,
                only_return_existing: false,
            },
            LetsEncrypt::Production.url(),
            None,
        )
        .await
        .map_err(|e| AppError::System(format!("Failed to create ACME account: {}", e)))?;

        // Save credentials for reuse
        if let Some(parent) = account_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(&credentials).map_err(|e| {
            AppError::System(format!("Failed to serialize ACME credentials: {}", e))
        })?;
        std::fs::write(account_path, json).map_err(|e| {
            AppError::System(format!("Failed to save ACME account credentials: {}", e))
        })?;

        tracing::info!(
            "Created new ACME account, saved to {}",
            account_path.display()
        );
        Ok(account)
    }

    fn cleanup_challenge_files(tokens: &[(PathBuf, String)]) {
        for (path, _) in tokens {
            let _ = std::fs::remove_file(path);
        }
    }

    /// Returns all Let's Encrypt certificates expiring within `days_before` days.
    pub async fn get_certificates_due_for_renewal(
        db: &DatabaseConnection,
        days_before: i64,
    ) -> AppResult<Vec<certificate::Model>> {
        let threshold = Utc::now() + chrono::Duration::days(days_before);
        let certs = CertEntity::find()
            .filter(certificate::Column::Provider.eq(CertProvider::LetsEncrypt))
            .filter(certificate::Column::ExpiresAt.lte(threshold))
            .all(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to query certificates: {}", e)))?;
        Ok(certs)
    }

    /// Collect all aliases currently configured on websites using this domain with SSL.
    pub async fn collect_domain_aliases(
        db: &DatabaseConnection,
        domain: &str,
    ) -> AppResult<Vec<String>> {
        let sites = WebsiteEntity::find()
            .filter(website::Column::HasSsl.eq(true))
            .filter(website::Column::PrimaryDomain.eq(domain))
            .all(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to query websites: {}", e)))?;

        let mut all_aliases = BTreeSet::new();
        for site in &sites {
            if let Ok(aliases) = serde_json::from_value::<Vec<String>>(site.aliases.clone()) {
                for alias in aliases {
                    let normalized = alias.trim().trim_end_matches('.').to_ascii_lowercase();
                    if !normalized.is_empty() && normalized != domain {
                        all_aliases.insert(normalized);
                    }
                }
            }
        }

        Ok(all_aliases.into_iter().collect())
    }

    /// Renew a single ACME certificate. Custom certificates cannot be renewed this way.
    pub async fn renew_certificate(
        db: &DatabaseConnection,
        cert: &certificate::Model,
    ) -> AppResult<certificate::Model> {
        match cert.provider {
            CertProvider::LetsEncrypt => {}
            CertProvider::Custom => {
                return Err(AppError::Validation(
                    "Custom certificates cannot be auto-renewed".to_string(),
                ));
            }
            CertProvider::ZeroSSL => {
                return Err(AppError::Validation(
                    "ZeroSSL certificates are not supported by auto-renewal".to_string(),
                ));
            }
        }

        let aliases = Self::collect_domain_aliases(db, &cert.domain).await?;

        tracing::info!(
            "Renewing certificate for '{}' (expires {}, aliases: {:?})",
            cert.domain,
            cert.expires_at,
            aliases,
        );

        Self::issue_certificate(db, &cert.domain, &aliases, None).await
    }

    /// Reload proxy containers for all SSL websites on the given domain.
    pub async fn reload_proxies_for_domain(
        db: &DatabaseConnection,
        domain: &str,
        app_manager: &ApplicationManager,
        docker: &DockerService,
    ) -> AppResult<()> {
        let sites = WebsiteEntity::find()
            .filter(website::Column::HasSsl.eq(true))
            .filter(website::Column::PrimaryDomain.eq(domain))
            .filter(
                Condition::any()
                    .add(website::Column::ServerType.eq(ServerType::Nginx))
                    .add(website::Column::ServerType.eq(ServerType::OpenResty)),
            )
            .all(db)
            .await
            .map_err(|e| AppError::System(format!("Failed to query websites: {}", e)))?;

        for site in &sites {
            if let Err(e) = ProxyConfigService::deploy(site, app_manager, docker).await {
                tracing::error!(
                    "Failed to reload proxy for website '{}' after certificate renewal: {}",
                    site.name,
                    e
                );
            }
        }

        Ok(())
    }
}

/// Check for expiring certificates and renew them. Intended to be called from a cron job
/// via the `renew-certs` CLI subcommand, or triggered manually via the API.
pub async fn run_renewal_cycle(
    db: &Arc<DatabaseConnection>,
    config: &Config,
    docker: Option<&DockerService>,
) -> AppResult<()> {
    let certs = CertificateService::get_certificates_due_for_renewal(db, 30).await?;

    if certs.is_empty() {
        tracing::info!("Certificate renewal check: no certificates due for renewal");
        return Ok(());
    }

    tracing::info!(
        "Certificate renewal check: {} certificate(s) due for renewal",
        certs.len()
    );

    let app_manager = ApplicationManager::new(PathBuf::from(&config.app_root_dir));
    let mut renewed_count = 0usize;
    let mut failed_domains = Vec::new();

    for cert in &certs {
        match CertificateService::renew_certificate(db, cert).await {
            Ok(renewed) => {
                renewed_count += 1;
                tracing::info!(
                    "Renewed certificate for '{}', new expiry: {}",
                    renewed.domain,
                    renewed.expires_at
                );
                if let Some(docker) = docker {
                    if let Err(e) = CertificateService::reload_proxies_for_domain(
                        db,
                        &renewed.domain,
                        &app_manager,
                        docker,
                    )
                    .await
                    {
                        tracing::error!(
                            "Failed to reload proxies after renewing '{}': {}",
                            renewed.domain,
                            e
                        );
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to renew certificate for '{}': {}", cert.domain, e);
                failed_domains.push(cert.domain.clone());
            }
        }
    }

    if !failed_domains.is_empty() {
        return Err(AppError::System(format!(
            "Certificate renewal finished with {} success(es) and {} failure(s): {}",
            renewed_count,
            failed_domains.len(),
            failed_domains.join(", ")
        )));
    }

    tracing::info!(
        "Certificate renewal check completed successfully: {} certificate(s) renewed",
        renewed_count
    );

    Ok(())
}
