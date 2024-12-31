use crate::context::AppContext;
use chrono::{Utc, Duration, DateTime};
use eyre::Result;
use redis::{Commands, Connection, ConnectionLike};
use slog::debug;

use super::models::Order;

pub trait OrderDao<'a> {
    fn new(context: &'a AppContext) -> Self;
    async fn get_order(&mut self, order_id: Vec<u8>) -> Result<Order>;
    async fn get_ready_orders(&mut self) -> Result<Vec<Order>>;
    async fn create_order(&mut self, order: &Order) -> Result<()>;
    async fn delete_order(&mut self, order_id: Vec<u8>) -> Result<()>;
}

pub struct OrderImpl<'a> {
    pub connection: Connection,
    pub context: &'a AppContext,
}

impl<'a> OrderDao<'a> for OrderImpl<'a> {
    fn new(context: &'a AppContext) -> Self {
        let client = redis::Client::open(context.config.redis_url.clone()).unwrap();
        let connection = client.get_connection().unwrap();
        // TODO Add logger or context?
        OrderImpl {
            connection,
            context,
        }
    }

    async fn get_order(&mut self, order_id: Vec<u8>) -> Result<Order> {
        let order_id = hex::encode(order_id);
        let redis_id = format!("order:{}", order_id);
        debug!(self.context.logger, "Getting order from Redis..."; "order" => format!("{:?}", redis_id));

        let order_json: String = redis::cmd("JSON.GET")
        .arg(redis_id) // Redis key
        .arg("$")       // JSON path (root object)
        .query(&mut self.connection)?;
        debug!(self.context.logger, "Got Order from Redis..."; "order" => format!("{:?}", order_json));

        let order: Vec<Order> = serde_json::from_str(&order_json)?;
        debug!(self.context.logger, "Correctly deserialized order!"; "order" => format!("{:?}", order));

        if order.len() != 1 {
            return Err(eyre::eyre!("Order not found"));
        } 
        return Ok(order[0].clone());

    }

    async fn create_order(&mut self, order: &Order) -> Result<()> {
        let order_json = serde_json::to_string(order)?;
        let order_id = hex::encode(&order.id);
        let redis_id = format!("order:{}", order_id);

        debug!(self.context.logger, "Creating Redis order..."; "order" => format!("{:?}", redis_id));
        let _: () = redis::cmd("JSON.SET")
        .arg(redis_id)
        .arg("$")
        .arg(order_json)
        .query(&mut self.connection)?;

        debug!(self.context.logger, "Create order succeeded!");
        return Ok(());
    }

    async fn delete_order(&mut self, order_id: Vec<u8>) -> Result<()> {
        let order_id = hex::encode(order_id);
        let redis_id = format!("order:{}", order_id);
        debug!(self.context.logger, "Deleting order from Redis..."; "order" => format!("{:?}", redis_id));

        // TODO Redis returns how many rows were effected, maybe check that?
        let result: Result<(), _> = self.connection.del(redis_id).map_err(|e| eyre::eyre!(e));

        debug!(self.context.logger, "Delete order succeeded!"; "result" => format!("{:?}", result));
        return result;
    }

    async fn get_ready_orders(&mut self) -> Result<Vec<Order>> {
        let now = Utc::now().timestamp();
        let mut cmd = redis::cmd("FT.SEARCH");
        cmd.arg("idx:orders")
            .arg(format!("@deadline:[{} +inf]", now));
    

        debug!(self.context.logger, "Getting ready orders from Redis..."; 
            "query" => format!("{:?}", String::from_utf8(cmd.get_packed_command()).unwrap()));

        let result: redis::Value = cmd.query(&mut self.connection)?; 

        result.into_sequence().unwrap().iter().for_each(|x| {
            debug!(self.context.logger, "Got result"; "result" => format!("{:?}", x));
        });
        // debug!(self.context.logger, "Get ready order succeeded!"; "results" => format!("{:?}", result));

        Err(eyre::eyre!("Not implemented"))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context::{AppConfig, AppContext},
        dao::{
            models::Order,
            redis::{OrderDao, OrderImpl},
        },
    };
    use chrono::Utc;
    use slog::{o, Drain};
    use std::ops::Add;

    #[tokio::test]
    #[ignore = "e2e"]
    async fn test_dao_create_get_delete() {
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

        let mut user_dao = OrderImpl::new(context);

        let mut expected_order = Order::default();
        expected_order.id = vec![1, 2, 3, 4];
        expected_order.deadline = Utc::now().add(chrono::Duration::days(2)).timestamp();
        expected_order.primary_filler_deadline = Utc::now().add(chrono::Duration::days(1)).timestamp();

        user_dao.create_order(&expected_order).await.unwrap();
        let actual_order = user_dao.get_order(expected_order.id.clone()).await.unwrap();
        // user_dao
        //     .delete_order(actual_order.id.clone())
        //     .await
        //     .unwrap();

        assert_eq!(expected_order.id, actual_order.id);
        assert_eq!(expected_order.user, actual_order.user);
        assert_eq!(expected_order.filler, actual_order.filler);
        assert_eq!(expected_order.call_data, actual_order.call_data);
        assert_eq!(expected_order.call_recipient, actual_order.call_recipient);
        assert_eq!(expected_order.destination_chain_selector, actual_order.destination_chain_selector);
        assert_eq!(expected_order.deadline, actual_order.deadline);
        // Add more field comparisons as necessary
    }

    #[tokio::test]
    #[ignore = "e2e"]
    async fn test_get_ready_orders() {
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

        let mut user_dao = OrderImpl::new(context);

        let mut expected_order = Order::default();
        expected_order.id = vec![1, 2, 3, 4];
        expected_order.deadline = Utc::now().add(chrono::Duration::days(2)).timestamp();
        expected_order.primary_filler_deadline = Utc::now().add(chrono::Duration::days(1)).timestamp();

        user_dao.create_order(&expected_order).await.unwrap();
        
        let orders = user_dao.get_ready_orders().await.unwrap();


    }
}
