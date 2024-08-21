// client docs - https://socket.io/docs/v4 

mod models;
mod socket;
mod scraping;

use std::{collections::HashMap, sync::{Arc, Mutex}};

use axum::{http::StatusCode, routing::get};
use dotenv::dotenv;
use socketioxide::SocketIo;
use tracing::info;

use crate::socket::socket_manager;

static TAG: &'static str = "CHATTER_BOX";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::default())?;
    dotenv().ok();

    // let pool = PgPoolOptions::new()
    //     .max_connections(32)
    //     .connect(env::var("CONNECTION_STRING")?.as_str()).await?;
    let (layer, io) = SocketIo::builder()
        .with_state(Arc::new(Mutex::new(HashMap::<String, u8>::new())))
        .build_layer();

    io.ns("/", socket_manager);

    
    let app = axum::Router::new()
        .route("/", get(|| async { (StatusCode::BAD_REQUEST, "Unable to upgrade request.") }))
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found") })
        .layer(layer);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();


    info!("Server started");
    axum::serve(listener, app).await.unwrap();

    return Ok(());
}
