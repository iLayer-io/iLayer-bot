use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Order::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Order::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Order::ChainId).big_unsigned().not_null())
                    .col(
                        ColumnDef::new(Order::OrderId)
                            .binary()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Order::User).binary().not_null())
                    .col(ColumnDef::new(Order::Filler).binary().not_null())
                    .col(
                        ColumnDef::new(Order::SourceChainSelector)
                            .binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Order::DestinationChainSelector)
                            .binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Order::Sponsored).boolean().not_null())
                    .col(
                        ColumnDef::new(Order::PrimaryFillerDeadline)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Order::Deadline).date_time().not_null())
                    .col(ColumnDef::new(Order::CallRecipient).binary())
                    .col(ColumnDef::new(Order::CallData).binary())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Order::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Order {
    Table,
    Id,
    ChainId,
    OrderId,
    User,
    Filler,
    SourceChainSelector,
    DestinationChainSelector,
    Sponsored,
    PrimaryFillerDeadline,
    Deadline,
    CallRecipient,
    CallData,
}
