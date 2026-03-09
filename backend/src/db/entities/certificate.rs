use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum CertProvider {
    #[sea_orm(string_value = "letsencrypt")]
    LetsEncrypt,
    #[sea_orm(string_value = "zerossl")]
    ZeroSSL,
    #[sea_orm(string_value = "custom")]
    Custom,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "certificates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub domain: String,
    pub provider: CertProvider,
    pub cert_path: String,
    pub key_path: String,
    pub expires_at: DateTimeUtc,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
