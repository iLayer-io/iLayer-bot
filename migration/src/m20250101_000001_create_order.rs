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
                    .col(ColumnDef::new(Order::User).string().not_null())
                    .col(ColumnDef::new(Order::Filler).string().not_null())
                    .col(ColumnDef::new(Order::SourceChainSelector).string().not_null())
                    .col(ColumnDef::new(Order::DestinationChainSelector).string().not_null())
                    .col(ColumnDef::new(Order::Sponsored).boolean().not_null())
                    .col(ColumnDef::new(Order::PrimaryFillerDeadline).date_time().not_null())
                    .col(ColumnDef::new(Order::Deadline).date_time().not_null())
                    .col(ColumnDef::new(Order::CallRecipient).string())
                    .col(ColumnDef::new(Order::CallData).string())
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
