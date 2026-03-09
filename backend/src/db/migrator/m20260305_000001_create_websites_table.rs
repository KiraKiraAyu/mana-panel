use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Websites::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Websites::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Websites::Name).string().not_null())
                    .col(
                        ColumnDef::new(Websites::PrimaryDomain)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Websites::Aliases)
                            .json()
                            .not_null()
                            .default("[]"),
                    )
                    .col(ColumnDef::new(Websites::ServerType).string().not_null())
                    .col(ColumnDef::new(Websites::ServerInstanceId).string().null())
                    .col(
                        ColumnDef::new(Websites::SiteTypes)
                            .json()
                            .not_null()
                            .default("[]"),
                    )
                    .col(ColumnDef::new(Websites::ProxyTargetType).string().null())
                    .col(ColumnDef::new(Websites::ProxyTargetUrl).string().null())
                    .col(ColumnDef::new(Websites::ProxyTargetAppId).string().null())
                    .col(
                        ColumnDef::new(Websites::ProxyTargetAppPort)
                            .integer()
                            .null(),
                    )
                    .col(ColumnDef::new(Websites::RootDir).string().null())
                    .col(
                        ColumnDef::new(Websites::Status)
                            .string()
                            .not_null()
                            .default("stopped"),
                    )
                    .col(ColumnDef::new(Websites::Error).string().null())
                    .col(
                        ColumnDef::new(Websites::HasSsl)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Websites::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Websites::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Websites::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Websites {
    Table,
    Id,
    Name,
    PrimaryDomain,
    Aliases,
    ServerType,
    ServerInstanceId,
    SiteTypes,
    ProxyTargetType,
    ProxyTargetUrl,
    ProxyTargetAppId,
    ProxyTargetAppPort,
    RootDir,
    Status,
    Error,
    HasSsl,
    CreatedAt,
    UpdatedAt,
}
