use crate::context::AppContext;
use eyre::Result;
use redis::{Commands, Connection};
use slog::info;

use super::models::Order;

pub trait OrderDao<'a> {
    fn new(context: &'a AppContext) -> Self;
    async fn get_order(&mut self, order_id: Vec<u8>) -> Result<Order>;
    async fn create_order(&mut self, order: &Order) -> Result<()>;
    async fn delete_order(&mut self, order_id: Vec<u8>) -> Result<()>;
}

pub struct UserImpl<'a> {
    pub connection: Connection,
    pub context: &'a AppContext,
}

impl<'a> OrderDao<'a> for UserImpl<'a> {
    fn new(context: &'a AppContext) -> Self {
        let client = redis::Client::open(context.config.redis_url.clone()).unwrap();
        let connection = client.get_connection().unwrap();
        // TODO Add logger or context?
        UserImpl { connection, context }
    }

    async fn get_order(&mut self, order_id: Vec<u8>) -> Result<Order> {
        let order_id = hex::encode(order_id);
        let redis_id = format!("order:{}", order_id);
        info!(self.context.logger, "Getting order from Redis..."; "order" => format!("{:?}", redis_id));

        let result = self.connection.get(redis_id).map_err(|e| eyre::eyre!(e));

        let order_json: String = result?;
        let order: Order = serde_json::from_str(&order_json)?;
        info!(self.context.logger, "Got Order from Redis!"; "order" => format!("{:?}", order));

        return Ok(order);
    }

    async fn create_order(&mut self, order: &Order) -> Result<()> {
        let order_json = serde_json::to_string(order)?;
        let order_id = hex::encode(&order.id);
        let redis_id = format!("order:{}", order_id);

        info!(self.context.logger, "Creating order from Redis..."; "order" => format!("{:?}", redis_id));
        let result = self
            .connection
            .set(redis_id, &order_json)
            .map_err(|e| eyre::eyre!(e));

        info!(self.context.logger, "Create order succeeded!"; "result" => format!("{:?}", result));
        return result;
    }

    async fn delete_order(&mut self, order_id: Vec<u8>) -> Result<()> {
        let order_id = hex::encode(order_id);
        let redis_id = format!("order:{}", order_id);
        info!(self.context.logger, "Deleting order from Redis..."; "order" => format!("{:?}", redis_id));

        // TODO Redis returns how many rows were effected, maybe check that?
        let result: Result<(), _> = self.connection.del(redis_id).map_err(|e| eyre::eyre!(e));

        info!(self.context.logger, "Delete order succeeded!"; "result" => format!("{:?}", result));
        return result;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context::{AppConfig, AppContext},
        dao::{
            models::Order,
            redis::{OrderDao, UserImpl},
        },
    };
    use slog::{o, Drain};

    #[tokio::test]
    #[ignore = "e2e"]
    async fn test_process_order_withdraw_log() {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = std::sync::Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();

        // TODO Take config from (test?) env vars
        let context = &AppContext {
            config: AppConfig {
                redis_url: "redis://localhost:6379".to_string(),
                rpc_url: Default::default(),
                ws_url: Default::default(),
                order_contract_address: Default::default(),
                from_block: Default::default(),
                redis_poll_interval: Default::default(),
            },
            logger: slog::Logger::root(drain, o!()),
        };

        let mut user_dao = UserImpl::new(context);

        let mut expected_order = Order::default();
        expected_order.id = vec![1, 2, 3, 4];

        user_dao.create_order(&expected_order).await.unwrap();
        let actual_order = user_dao.get_order(expected_order.id.clone()).await.unwrap();
        user_dao
            .delete_order(actual_order.id.clone())
            .await
            .unwrap();

        assert_eq!(expected_order, actual_order);
    }
}
