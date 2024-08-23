// client docs - https://socket.io/docs/v4 

mod error;
mod socket;
mod scraping;

use std::{collections::HashMap, sync::{Arc, Mutex}};

use axum::{http::StatusCode, routing::get};
use dotenv::dotenv;
use headless_chrome::{Browser, LaunchOptions};
use socketioxide::SocketIo;
use tracing::info;

use crate::socket::socket_manager;
pub use crate::error::Result;
pub use crate::error::Error;

static _TAG: &'static str = "CHATTER_BOX";


#[tokio::main]
async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::default())?;
    dotenv().ok();

    // let pool = PgPoolOptions::new()
    //     .max_connections(32)
    //     .connect(env::var("CONNECTION_STRING")?.as_str()).await?;
    let browser = Browser::new(LaunchOptions {
        headless: false,
        ..Default::default()
    })?;
    let (layer, io) = SocketIo::builder()
        .with_state(Arc::new(Mutex::new(HashMap::<String, u8>::new())))
        .with_state(Arc::new(browser))
        .build_layer();

    io.ns("/", socket_manager);

    let app = axum::Router::new()
        .route("/", get(|| async { (StatusCode::BAD_REQUEST, "Unable to upgrade request.") }))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found") })
        .layer(layer);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;


    info!("Server started");
    axum::serve(listener, app).await?;

    return Ok(());
}
