use std::{collections::HashMap, sync::{Arc, Mutex}, time::Duration};

use socketioxide::extract::{Data, State, SocketRef};
use tokio::time::sleep;
use tracing::{info, error};

use crate::scraping::kick::{go_to, scrape};


 pub async fn socket_manager(socket: SocketRef) {
    info!("socket connected: {}", socket.id);

    socket.on("join", join);
    socket.on("leave", leave);
    socket.on_disconnect(on_disconnect);
}


fn join(
    socket: SocketRef, 
    Data(streamer): Data<String>, 
    State(state): State<Arc<Mutex<HashMap<String, u8>>>>,
    State(browser): State<Arc<headless_chrome::Browser>>
) {
    info!("Received join: {:?}", streamer);

    let browser = Arc::clone(&browser);
    let rooms = Arc::clone(&state);
    let mut rooms = rooms.lock().unwrap();
    let streamer = streamer.to_owned();

    if !rooms.contains_key(&streamer) {
        let tab = match go_to(browser, &streamer) {
            Ok(t) => t,
            Err(e) => {
                error!("ON GO TO: {}", e);
                return;
            } 
        };

        socket.join(streamer.to_owned()).unwrap();
        rooms.insert(streamer.to_owned(), 1);
        tokio::task::spawn(async move {
            loop {
                // underscore to silence warning
                let mut _has_connections = false;
                {
                    let rooms = Arc::clone(&state);
                    let rooms = rooms.lock().unwrap();
                    _has_connections =  rooms.contains_key(&streamer);
                }
                if !_has_connections { break; }
                let messages = match scrape(&tab) {
                    Ok(msgs) => msgs,
                    Err(e) => {
                        error!("ON SCRAPE: {e}");
                        break;
                    }
                };

                for msg in messages {
                    info!(
                        "EMIT: {} {} {} {}", 
                        msg.username,
                        msg.user_color,
                        msg.content.clone().unwrap_or("".to_string()),
                        msg.emote_html.clone().unwrap_or("".to_string()),
                    ); 
                    socket
                        .within(streamer.to_owned())
                        .emit("chat", msg)
                        .unwrap();
                }
                sleep(Duration::new(1,0)).await;
            }
        });
    } else {
        socket.join(streamer.to_owned()).unwrap();
        let listeners = rooms.get_mut(&streamer).unwrap();
        *listeners += 1;

        info!("{streamer}: {}", listeners); 
    }
}

fn leave(
    socket: SocketRef, 
    Data(streamer): Data<String>,
    State(state): State<Arc<Mutex<HashMap<String, u8>>>>
) {
    info!("Received leave: {:?}", streamer);

    let rooms = Arc::clone(&state);
    let mut rooms = rooms.lock().unwrap();

    for room in socket.rooms().unwrap() {
        if room == streamer {
            let Some(listeners) = rooms.get_mut(&streamer) else {
                break;
            };
            *listeners -= 1;

            if *listeners <= 0 {
                rooms.remove(&streamer);
            }
        }
    }
    socket.leave(streamer).unwrap();
}

fn on_disconnect(
    socket: SocketRef, 
    State(state): State<Arc<Mutex<HashMap<String, u8>>>>
) {
    info!("{} disconnected", socket.id);

    let rooms = Arc::clone(&state);
    let mut rooms = rooms.lock().unwrap();

    for room in socket.rooms().unwrap() {
        let Some(listeners) = rooms.get_mut(&room.to_string()) else {
            socket.leave(room.to_owned()).unwrap();
            continue;
        };
        *listeners -= 1;

        if *listeners <= 0 {
            rooms.remove(&room.to_string());
        }
    }
    socket.leave_all().unwrap();
}
