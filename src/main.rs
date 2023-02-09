mod routes;

use opentelemetry::sdk::export::trace::stdout;
use std::path::Path;
use anyhow::Context;
use opentelemetry::sdk::trace::Tracer;
use tracing::{error, info, span, Span, Subscriber};
use tracing_error::ErrorLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use crate::routes::build_routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    init_server().await?;
    Ok(())
}

#[tracing::instrument]
async fn init_server() -> Result<(), anyhow::Error> {

    let asset_dir = std::env::var("ASSET_DIR").unwrap_or_else(|_| String::from("/assets"));
    let index_path = format!("{asset_dir}/index.html");
    info!("using asset dir: {asset_dir}");
    if !Path::new(&index_path).exists() {
        anyhow::bail!("index.html not found at {index_path}");
    }

    let port = std::env::var("PORT").unwrap_or_else(|_| String::from("8080"));
    info!("using port: {port}");
    let port = port.parse::<u16>().context("failed to parse port")?;

    let routes = build_routes(asset_dir, index_path);

    let (_, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], port), async move {
            tracing::log::info!("waiting for shutdown signal");
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen to shutdown signal");
            tracing::log::info!("got shutdown signal");
        });
    match tokio::join!(tokio::task::spawn(server)).0 {
        Ok(()) => info!("served"),
        Err(e) => error!("ERROR: Thread join error {}", e),
    }

    Ok(())
}

fn init_opentelemetry<S>() -> Option<OpenTelemetryLayer<S,Tracer>>
    where
        S : Subscriber + for<'span> LookupSpan<'span>
{
    let tracer = stdout::new_pipeline().install_simple();

    // Create a tracing subscriber with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    Some(telemetry)
}

fn init_tracing() {
    let subscriber = tracing_subscriber::Registry::default()
        .with(ErrorLayer::default())
        .with(init_opentelemetry());

    tracing::subscriber::set_global_default(subscriber).expect("unable to initialize tracing");
}
