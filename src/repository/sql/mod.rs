use eyre::{Ok, Result};
use sea_orm::*;
use ::entity::order::{self, ActiveModel, Entity as Order};

use crate::context::AppContext;

pub struct OrderRepository<'a> {
    pub _context: &'a AppContext,
    pub connection: sea_orm::DatabaseConnection,
}
pub async fn new<'a>(context: &'a AppContext) -> Result<OrderRepository<'a>> {
    let connection: sea_orm::DatabaseConnection = Database::connect(context.config.postgres_url.clone()).await?;
    // TODO Use logger iof context?
    Ok(OrderRepository {
        connection,
        _context:context,
    })
}

impl<'a> OrderRepository<'a> {

    pub async fn get_order(&mut self, order_id: Vec<u8>) -> Result<order::Model> {
        let order = Order::find()
        .filter(order::Column::OrderId.eq(order_id))
        .one(&self.connection)
        .await?;
        order.ok_or(eyre::eyre!("Order not found"))
    }

    pub async fn create_order(&mut self, order: &ActiveModel) -> Result<()> {
        order::Entity::insert(order.clone()).exec(&self.connection).await?;
        Ok(())
    }

    pub async fn delete_order(&mut self, order_id: Vec<u8>) -> Result<()> {
        order::Entity::delete_many()
            .filter(order::Column::OrderId.eq(order_id))
            .exec(&self.connection)
            .await?;
        Ok(())
    }

    pub async fn get_ready_orders(&mut self) -> Result<Vec<order::Model>> {
        let ready_orders = Order::find()
        .filter(order::Column::Deadline.gt(chrono::Utc::now().naive_utc()))
        .all(&self.connection)
        .await?;
        Ok(ready_orders)
    }
}


#[cfg(test)]
mod tests {
    use ::entity::order;
    use eyre::Ok;
    use slog::o;
    use slog::Drain;
    use crate::context::{AppConfig, AppContext};
    use sea_orm::*;


   #[tokio::test]
    async fn test_example_1() -> eyre::Result<()> {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = std::sync::Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();

        
        let context = &AppContext {
            config: AppConfig {
                postgres_url: "postgres://postgres:postgres@localhost:5432/bot".to_string(),
                ..Default::default()
            },
            logger: slog::Logger::root(drain, o!()),
        };

        let expected_order = &order::ActiveModel {
            user: ActiveValue::set("user".as_bytes().to_owned()),
            order_id: ActiveValue::set("order_id".as_bytes().to_owned()),
            filler: ActiveValue::set("filler".as_bytes().to_owned()),
            source_chain_selector: ActiveValue::set("source_chain_selector".as_bytes().to_owned()),
            destination_chain_selector: ActiveValue::set("destination_chain_selector".as_bytes().to_owned()),
            sponsored: ActiveValue::set(false),
            primary_filler_deadline: ActiveValue::set(chrono::Utc::now().naive_utc()),
            deadline: ActiveValue::set(chrono::Utc::now().naive_utc()),
            id: ActiveValue::NotSet,
            call_recipient: ActiveValue::NotSet,
            call_data: ActiveValue::NotSet,
        };
        
        let mut order_dao = super::new(context).await?;
        
        order_dao.delete_order("order_id".as_bytes().to_vec()).await?;
        order_dao.create_order(expected_order).await?;
        let actual_order = order_dao.get_order("order_id".as_bytes().to_vec()).await?;
        order_dao.delete_order("order_id".as_bytes().to_vec()).await?;
        
        assert_eq!(expected_order.order_id.clone().unwrap(), actual_order.order_id); 
        Ok(())
    }

}
