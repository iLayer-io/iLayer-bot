use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use eyre::{Result};
use crate::orm::schema::orders;
use crate::orm::models::Order;

pub trait OrderDao {
    fn getOrder(&mut self, order_id: Vec<u8>) -> Result<Order>;
}

pub struct UserImpl {
    pub conn: PgConnection,
}

impl OrderDao for UserImpl {
    fn getOrder(&mut self, order_id: Vec<u8>) -> Result<Order> {
        return orders::dsl::orders
        .filter(orders::dsl::order_id.eq(order_id))
        .first::<Order>(&mut self.conn).map_err(|e| e.into());
    }
 }


 
#[cfg(test)]
mod tests {
    use crate::context::{self, AppConfig, AppContext};
    use alloy::primitives::{Log as PrimitivesLog, LogData};
    use alloy::rpc::types::Log;
    use diesel::{Connection, PgConnection};
    use slog::{o, Drain};

    #[tokio::test]
    #[ignore]
    async fn test_process_order_withdraw_log() {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = std::sync::Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();

        let context = AppContext {
            config: AppConfig {
                database_url: "postgres://postgres:postgres@localhost:5432/solver".to_string(),
                rpc_url: "http://localhost:8545".to_string(),
                ws_url: "ws://localhost:8545".to_string(),
                order_contract_address: "0x".to_string(),
                from_block: Some(0),
            },
            logger: slog::Logger::root(drain, o!()),
        };

        let mut _conn = PgConnection::establish(&context.config.database_url).unwrap();

    }
}
