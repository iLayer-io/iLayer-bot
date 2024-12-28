use crate::{context::AppContext, orm::models::Order};
use eyre::Result;
use redis::{Commands, Connection};
use slog::info;

pub trait OrderDao<'a> {
    fn new(conn: Connection, context: &'a AppContext) -> Self;
    async fn get_order(&mut self, order_id: Vec<u8>) -> Result<Order>;
    async fn create_order(&mut self, order: Order) -> Result<()>;
    async fn delete_order(&mut self, order_id: Vec<u8>) -> Result<()>;
}

pub struct UserImpl<'a> {
    pub conn: Connection,
    pub context: &'a AppContext,
}

impl<'a> OrderDao<'a> for UserImpl<'a> {
    fn new(conn: Connection, context: &'a AppContext) -> Self {
        // TODO Add logger or context?
        UserImpl { conn, context }
    }

    async fn get_order(&mut self, order_id: Vec<u8>) -> Result<Order> {
        let order_id = hex::encode(order_id);
        let redis_id = format!("order:{}", order_id);

        let result = self
            .conn
            .get(redis_id)
            .map_err(|e| eyre::eyre!(e));

        let order_json: String = result?;
        let order: Order = serde_json::from_str(&order_json)?;
        return Ok(order);
    }

    async fn create_order(&mut self, order: Order) -> Result<()> {
        let order_json = serde_json::to_string(&order)?;
        let order_id = hex::encode(order.id);
        let redis_id = format!("order:{}", order_id);

        info!(self.context.logger, "creating order on redis..."; "order" => format!("{:?}", redis_id));
        let result = self
            .conn
            .set(redis_id, &order_json)
            .map_err(|e| eyre::eyre!(e));

        info!(self.context.logger, "create order succeeded!"; "result" => format!("{:?}", result));
        return result;
    }

    async fn delete_order(&mut self, order_id: Vec<u8>) -> Result<()> {
        let order_id = hex::encode(order_id);
        let redis_id = format!("order:{}", order_id);
        // TODO Redis returns how many rows were effected, maybe check that? 
        let result: Result<(), _> = self
            .conn
            .del(redis_id)
            .map_err(|e| eyre::eyre!(e));

        return result;
    }
}

#[cfg(test)]
mod tests {
    use crate::context::{AppConfig, AppContext};
    use slog::{o, Drain};

    #[tokio::test]
    #[ignore = "e2e"]
    async fn test_process_order_withdraw_log() {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = std::sync::Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();

        let context = AppContext {
            config: AppConfig {
                redis_url: "postgres://postgres:postgres@localhost:5432/solver".to_string(),
                rpc_url: "http://localhost:8545".to_string(),
                ws_url: "ws://localhost:8545".to_string(),
                order_contract_address: "0x".to_string(),
                from_block: Some(0),
            },
            logger: slog::Logger::root(drain, o!()),
        };
    }
}
