use crate::config::Config;

pub mod proxy;
pub mod config;

#[tokio::main]
async fn main() {
    let res = run().await;
    match res {
        Err(err) => log::error!("{:?}", err),
        Ok(_) => log::info!("Done"),
    }
}

async fn run() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    pretty_env_logger::env_logger::builder().init();

    let config_filename = std::env::var("HDP_CONFIG").unwrap_or("./config.yaml".to_string());
    log::debug!("Loading config {:?}...", &config_filename);
    let f = std::fs::File::open(config_filename)?;
    let config: Config = serde_yaml::from_reader(f)?;

    log::info!("Starting proxy on {}:{}...", &config.server.host, config.server.port);

    proxy::server::run(config).await?;

    Ok(())
}
