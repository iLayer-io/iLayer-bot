use extension::postgres::Type;
use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        match db.get_database_backend() {
            DbBackend::MySql | DbBackend::Sqlite => {}
            DbBackend::Postgres => {
                manager
                    .create_type(
                        Type::create()
                            .as_enum(OrderStatus::Enum)
                            .values([
                                OrderStatus::Created,
                                OrderStatus::Filled,
                                OrderStatus::Withdrawn,
                            ])
                            .to_owned(),
                    )
                    .await?;
            }
        }

        manager
            .alter_table(
                sea_query::Table::alter()
                    .table(crate::m20250101_000001_create_order::Order::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("order_status"))
                            .custom(OrderStatus::Enum)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        match db.get_database_backend() {
            DbBackend::MySql | DbBackend::Sqlite => {}
            DbBackend::Postgres => {
                manager
                    .drop_type(Type::drop().name(OrderStatus::Enum).to_owned())
                    .await?;
            }
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum OrderStatus {
    #[sea_orm(iden = "order_status")]
    Enum,
    #[sea_orm(iden = "Created")]
    Created,
    #[sea_orm(iden = "Filled")]
    Filled,
    #[sea_orm(iden = "Withdrawn")]
    Withdrawn,
}
