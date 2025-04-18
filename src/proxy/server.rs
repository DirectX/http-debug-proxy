use std::{sync::atomic::AtomicUsize, time::Duration};

use actix_web::{http::KeepAlive, web::Data, App, HttpServer};

use crate::config::Config;

use super::handlers::index;

pub async fn run(config: Config) -> std::io::Result<()> {
    let host = config.server.host.to_owned();
    let port = config.server.port;

    let app_config = Data::new(config.clone());

    if config.upstreams.len() < 1 {
        panic!("No upstreams defined");
    }

    if config.upstreams.len() > 1 {
        if let Some(default_upstream) = config.default_upstream {
            if !config.upstreams.contains_key(&default_upstream) {
                let available_upstreams = config.upstreams.iter().map(|(k, _)| format!("\"{}\"", k)).collect::<Vec<String>>().join(", ");
                panic!("Default upstream \"{}\" doesn't match to any of upstreams ({})", default_upstream, available_upstreams);
            }
        } else {
            panic!("Default upstream is not defined");
        }
    }

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let app_client = Data::new(client);

    let counter = AtomicUsize::new(0usize);
    let app_counter = Data::new(counter);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_config.clone())
            .app_data(app_client.clone())
            .app_data(app_counter.clone())
            .service(index)
    })
    .keep_alive(KeepAlive::Timeout(Duration::from_secs(75)));

    server.bind((host, port))?.run().await
}
