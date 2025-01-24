use ::entity::{
    order::{self, ActiveModel, Entity as Order},
    sea_orm_active_enums::OrderStatus,
};
use eyre::{Ok, Result};
use sea_orm::*;

pub struct OrderRepository {
    pub connection: sea_orm::DatabaseConnection,
}

impl OrderRepository {
    pub async fn new(postgres_url: String) -> Result<Self> {
        let connection: sea_orm::DatabaseConnection = Database::connect(postgres_url).await?;
        Ok(OrderRepository { connection })
    }

    pub async fn get_order(&self, order_id: Vec<u8>) -> Result<order::Model> {
        let order = Order::find()
            .filter(order::Column::OrderId.eq(order_id))
            .one(&self.connection)
            .await?;
        order.ok_or(eyre::eyre!("Order not found"))
    }

    pub async fn create_order(&self, order: &ActiveModel) -> Result<i32> {
        let insert_result = order::Entity::insert(order.clone())
            .exec(&self.connection)
            .await?;
        Ok(insert_result.last_insert_id)
    }

    #[allow(dead_code)]
    pub async fn delete_order(&self, order_id: Vec<u8>) -> Result<()> {
        order::Entity::delete_many()
            .filter(order::Column::OrderId.eq(order_id))
            .exec(&self.connection)
            .await?;
        Ok(())
    }

    pub async fn update_order_status(
        &self,
        order_id: Vec<u8>,
        order_status: OrderStatus,
    ) -> Result<()> {
        let mut order: ActiveModel = Order::find()
            .filter(order::Column::OrderId.eq(order_id.clone()))
            .one(&self.connection)
            .await?
            .ok_or(eyre::eyre!("Order not found"))?
            .into();

        order.order_status = ActiveValue::Set(order_status);

        order.update(&self.connection).await?;
        Ok(())
    }

    pub async fn get_ready_orders(&self, chain_id: u64) -> Result<Vec<order::Model>> {
        let ready_orders = Order::find()
            // .filter(order::Column::PrimaryFillerDeadline.gt(chrono::Utc::now().naive_utc()))
            // .filter(order::Column::Deadline.gt(chrono::Utc::now().naive_utc()))
            .filter(order::Column::ChainId.eq(chain_id))
            .filter(order::Column::OrderStatus.eq(OrderStatus::Created))
            .all(&self.connection)
            .await?;
        Ok(ready_orders)
    }
}

#[cfg(test)]
mod tests {
    use super::OrderRepository;
    use ::entity::{order, sea_orm_active_enums::OrderStatus};
    use eyre::Ok;
    use sea_orm::*;

    #[tokio::test]
    #[ignore]
    async fn test_create_get_delete() -> eyre::Result<()> {
        let postgres_url = "postgres://postgres:postgres@localhost:5432/bot".to_string();

        let expected_order = &order::ActiveModel {
            user: ActiveValue::set("user".as_bytes().to_owned()),
            chain_id: ActiveValue::set(1),
            order_id: ActiveValue::set("order_id".as_bytes().to_owned()),
            filler: ActiveValue::set("filler".as_bytes().to_owned()),
            source_chain_selector: ActiveValue::set("source_chain_selector".as_bytes().to_owned()),
            destination_chain_selector: ActiveValue::set(
                "destination_chain_selector".as_bytes().to_owned(),
            ),
            sponsored: ActiveValue::set(false),
            primary_filler_deadline: ActiveValue::set(chrono::Utc::now().naive_utc()),
            deadline: ActiveValue::set(chrono::Utc::now().naive_utc()),
            id: ActiveValue::NotSet,
            call_recipient: ActiveValue::NotSet,
            call_data: ActiveValue::NotSet,
            order_status: ActiveValue::Set(OrderStatus::Created),
        };

        let order_repository = OrderRepository::new(postgres_url).await?;

        order_repository
            .delete_order("order_id".as_bytes().to_vec())
            .await?;
        order_repository.create_order(expected_order).await?;
        let actual_order = order_repository
            .get_order("order_id".as_bytes().to_vec())
            .await?;
        order_repository
            .delete_order("order_id".as_bytes().to_vec())
            .await?;

        assert_eq!(
            expected_order.order_id.clone().unwrap(),
            actual_order.order_id
        );
        Ok(())
    }
}
