use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum ServerType {
    #[sea_orm(string_value = "nginx")]
    Nginx,
    #[sea_orm(string_value = "caddy")]
    Caddy,
    #[sea_orm(string_value = "openresty")]
    OpenResty,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum SiteType {
    #[sea_orm(string_value = "reverse_proxy")]
    ReverseProxy,
    #[sea_orm(string_value = "static")]
    Static,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum ProxyTargetType {
    #[sea_orm(string_value = "url")]
    Url,
    #[sea_orm(string_value = "application")]
    Application,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
#[serde(rename_all = "snake_case")]
pub enum WebsiteStatus {
    #[sea_orm(string_value = "running")]
    Running,
    #[sea_orm(string_value = "stopped")]
    Stopped,
    #[sea_orm(string_value = "error")]
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "websites")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub primary_domain: String,
    pub aliases: serde_json::Value,
    pub server_type: ServerType,
    pub server_instance_id: Option<String>,
    pub site_type: SiteType,
    pub proxy_target_type: Option<ProxyTargetType>,
    pub proxy_target_url: Option<String>,
    pub proxy_target_app_id: Option<String>,
    pub proxy_target_app_port: Option<i32>,
    pub root_dir: Option<String>,
    pub status: WebsiteStatus,
    pub error: Option<String>,
    pub has_ssl: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
