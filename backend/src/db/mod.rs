pub mod entities;
pub mod migrator;

use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

pub async fn init_database(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    
    // Run migrations
    migrator::Migrator::up(&db, None).await?;
    
    Ok(db)
}
