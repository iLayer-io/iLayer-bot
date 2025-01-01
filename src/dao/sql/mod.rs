use entity::order::Entity as Order;
use eyre::Result;
use slog::debug;

use crate::context::AppContext;


pub trait OrderDao<'a> {
    fn new(context: &'a AppContext) -> Self;
    async fn get_order(&mut self, order_id: Vec<u8>) -> Result<Order>;
    async fn get_ready_orders(&mut self) -> Result<Vec<Order>>;
    async fn create_order(&mut self, order: &Order) -> Result<()>;
    async fn delete_order(&mut self, order_id: Vec<u8>) -> Result<()>;
}

pub struct OrderImpl<'a> {
    pub context: &'a AppContext,
}

impl<'a> OrderDao<'a> for OrderImpl<'a> {
    fn new(context: &'a AppContext) -> Self {
        // TODO Use logger iof context?
        OrderImpl {
            context,
        }
    }

    async fn get_order(&mut self, _order_id: Vec<u8>) -> Result<Order> {
        todo!()
    }

    async fn create_order(&mut self, _order: &Order) -> Result<()> {
        todo!();
        return Ok(());
    }

    async fn delete_order(&mut self, _order_id: Vec<u8>) -> Result<()> {
        todo!()
    }

    async fn get_ready_orders(&mut self) -> Result<Vec<Order>> {
        todo!()
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
                rpc_url: Default::default(),
                ws_url: Default::default(),
                order_contract_address: Default::default(),
                from_block: Default::default(),
                redis_poll_interval: Default::default(),
            },
            logger: slog::Logger::root(drain, o!()),
        };

        let db = Database::connect(context.config.postgres_url.clone()).await?;

        let _o = order::ActiveModel {
            user: ActiveValue::set("user".to_owned()),
            filler: ActiveValue::set("filler".to_owned()),
            source_chain_selector: ActiveValue::set("source_chain_selector".to_owned()),
            destination_chain_selector: ActiveValue::set("destination_chain_selector".to_owned()),
            sponsored: ActiveValue::set(false),
            primary_filler_deadline: ActiveValue::set(chrono::Utc::now().naive_utc()),
            deadline: ActiveValue::set(chrono::Utc::now().naive_utc()),
            id: ActiveValue::NotSet,
            call_recipient: ActiveValue::NotSet,
            call_data: ActiveValue::NotSet,
        };
        order::Entity::insert(_o).exec(&db).await?;
        
        Ok(())
    }

}
