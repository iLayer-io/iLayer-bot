use ::entity::block_checkpoint::{self, ActiveModel, Entity as LastProcessedBlock};
use eyre::{Ok, Result};
use sea_orm::*;

pub struct BlockCheckpointRepository {
    pub connection: sea_orm::DatabaseConnection,
}

impl BlockCheckpointRepository {
    pub async fn new(postgres_url: String) -> Result<Self> {
        let connection: sea_orm::DatabaseConnection = Database::connect(postgres_url).await?;
        Ok(Self { connection })
    }

    pub async fn get_last_block_checkpoint(&self) -> Result<Option<block_checkpoint::Model>> {
        Ok(LastProcessedBlock::find()
            .order_by_desc(block_checkpoint::Column::Height)
            .one(&self.connection)
            .await?)
    }

    pub async fn create_block_checkpoint(&self, chain_id: u64, height: u64) -> Result<()> {
        let block = ActiveModel {
            chain_id: Set(chain_id as i64),
            height: Set(height as i64),
            ..Default::default()
        };
        block_checkpoint::Entity::insert(block)
            .exec(&self.connection)
            .await?;
        Ok(())
    }
}
