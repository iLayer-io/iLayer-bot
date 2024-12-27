use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};
use eyre::Result;
use crate::orm::schema::orders;
use crate::orm::models::{NewOrder, Order};

pub trait OrderDao {
    fn get_order(&mut self, order_id: Vec<u8>) -> Result<Order>;
    fn create_order(&mut self, order: NewOrder) -> Result<i32>;
}

pub struct UserImpl {
    pub conn: PgConnection,
}

impl OrderDao for UserImpl {
    fn get_order(&mut self, order_id: Vec<u8>) -> Result<Order> {
        let result = orders::dsl::orders
        .filter(orders::dsl::order_id.eq(order_id))
        .select(Order::as_select())
        .first::<Order>(&mut self.conn).map_err(|e| e.into());
    
        return result;
    }

    fn create_order(&mut self, order: NewOrder) -> Result<i32> {
        let result = diesel::insert_into(orders::table)
            .values(&order)
            .returning(orders::dsl::id) 
            .get_result::<i32>(&mut self.conn)
            .map_err(|e| e.into());

        return result;
    }
 }


 
#[cfg(test)]
mod tests {
    use crate::context::{AppConfig, AppContext};
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
