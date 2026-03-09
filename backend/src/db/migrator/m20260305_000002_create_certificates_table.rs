use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Certificates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Certificates::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Certificates::Domain).string().not_null())
                    .col(ColumnDef::new(Certificates::Provider).string().not_null())
                    .col(ColumnDef::new(Certificates::CertPath).string().not_null())
                    .col(ColumnDef::new(Certificates::KeyPath).string().not_null())
                    .col(
                        ColumnDef::new(Certificates::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Certificates::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Certificates::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Certificates::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Certificates {
    Table,
    Id,
    Domain,
    Provider,
    CertPath,
    KeyPath,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}
