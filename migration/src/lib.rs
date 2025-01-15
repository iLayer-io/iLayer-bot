pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_order;
mod m20250101_000002_create_block_checkpoint;
mod m20250101_000003_create_order_status_enum_and_column;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_order::Migration),
            Box::new(m20250101_000002_create_block_checkpoint::Migration),
            Box::new(m20250101_000003_create_order_status_enum_and_column::Migration),
        ]
    }
}
