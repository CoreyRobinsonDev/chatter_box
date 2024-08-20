// client docs - https://socket.io/docs/v4 

mod scraping;
mod models;

use std::{sync::{Arc, Mutex}, time::Duration};
use std::env;

use axum::{http::StatusCode, routing::get};
use dotenv::dotenv;
use models::{Chatroom, Server};
use socketioxide::{extract::{Data, SocketRef, State}, SocketIo};
use sqlx::postgres::PgPoolOptions;
use tokio::time::sleep;
use tracing::{info, error};

static TAG: &'static str = "CHATTER_BOX";

fn join(
    socket: SocketRef, 
    Data(chatroom): Data<String>, 
    State(s): State<Arc<Mutex<Server>>>,
) {
    info!("Received join: {:?}", chatroom);

    let server = Arc::clone(&s);
    let mut server = server.lock().unwrap();
    let room_name = chatroom.to_owned();

    if !server.contains(&chatroom) {
        socket.join(chatroom.to_owned()).unwrap();
        server.chatrooms.push(Chatroom::new(room_name));
        tokio::task::spawn(async move {
            loop {
                let mut has_connections = false;
                {
                    let server = Arc::clone(&s);
                    let server = server.lock().unwrap();
                    has_connections =  server.contains(&chatroom);
                }
                if has_connections  {
                    info!("hello from {}", chatroom); 
                    socket.within(chatroom.to_owned()).emit("chat", format!("hello from {chatroom}")).unwrap();

                    sleep(Duration::new(1,0)).await;
                } else { break; };
            }
        });
    } else {
        socket.join(chatroom.to_owned()).unwrap();
        let Some(room) = server.find(&chatroom) else {
            socket.leave(chatroom.to_owned()).unwrap();
            return;
        };

        room.listeners += 1;

        info!("{chatroom}: {}", room.listeners); 
    }

}

async fn kick_chat(socket: SocketRef) {
    info!("socket connected: {}", socket.id);

    socket.on("join", join);

    socket.on("leave", |
        socket: SocketRef, Data::<String>(chatroom),
        State(server): State<Arc<Mutex<Server>>>| {
        info!("Received leave: {:?}", chatroom);

        let server = Arc::clone(&server);
        let mut server = server.lock().unwrap();

        for room in socket.rooms().unwrap() {
            if room == chatroom.to_string() {
                let Some(room) = server.find(&chatroom.to_string()) else {
                    socket.leave(chatroom.to_owned()).unwrap();
                    return;
                };

                room.listeners -= 1;

                if room.listeners <= 0 {
                    server.remove(&chatroom);
                }
            }
        }
        socket.leave(chatroom).unwrap();
    });

    socket.on_disconnect(|socket: SocketRef, 
        State(server): State<Arc<Mutex<Server>>>| {
        info!("{} disconnected", socket.id);

        let server = Arc::clone(&server);
        let mut server = server.lock().unwrap();

        for chatroom in socket.rooms().unwrap() {
            let Some(room) = server.find(&chatroom.to_string()) else {
                socket.leave(chatroom.to_owned()).unwrap();
                return;
            };

            room.listeners -= 1;

            if room.listeners <= 0 {
                server.remove(chatroom);
            }
        }
        socket.leave_all().unwrap();
    });

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::default())?;
    dotenv().ok();

    // let pool = PgPoolOptions::new()
    //     .max_connections(32)
    //     .connect(env::var("CONNECTION_STRING")?.as_str()).await?;
    let (layer, io) = SocketIo::builder()
        .with_state(Arc::new(Mutex::new(Server::new())))
        .build_layer();

    io.ns("/", kick_chat);

    
    let app = axum::Router::new()
        .route("/", get(|| async { (StatusCode::BAD_REQUEST, "Unable to upgrade request.") }))
        .route("/kick/chat", get(|| async { (StatusCode::BAD_REQUEST, "Unable to upgrade request.") }))
        .route("/twitch/chat", get(|| async { (StatusCode::BAD_REQUEST, "Unable to upgrade request.") }))
        .layer(layer);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();


    info!("Server started");
    axum::serve(listener, app).await.unwrap();

    return Ok(());
}
