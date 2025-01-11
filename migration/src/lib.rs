pub use sea_orm_migration::prelude::*;

mod m20250101_000001_create_order;
mod m20250101_000002_create_block_checkpoint;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_order::Migration),
            Box::new(m20250101_000002_create_block_checkpoint::Migration),
        ]
    }
}
