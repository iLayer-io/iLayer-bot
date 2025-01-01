

#[cfg(test)]
mod tests {
    use eyre::Ok;
    use sea_orm::Database;
    use slog::o;
    use slog::Drain;
    use crate::context::{AppConfig, AppContext};


    #[tokio::test]
    async fn test_example_1() -> eyre::Result<()> {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = std::sync::Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();

        
        let context = &AppContext {
            config: AppConfig {
                postgres_url: "postgres://postgres:postgres@localhost:5432/postgres".to_string(),
                redis_url: "redis://localhost:6379".to_string(),
                rpc_url: Default::default(),
                ws_url: Default::default(),
                order_contract_address: Default::default(),
                from_block: Default::default(),
                redis_poll_interval: Default::default(),
            },
            logger: slog::Logger::root(drain, o!()),
        };

        let _db = Database::connect(context.config.postgres_url.clone()).await?;

        Ok(())
    }

}
