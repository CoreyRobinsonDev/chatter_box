use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}, time::Duration};

use socketioxide::extract::{Data, State, SocketRef};
use tokio::time::sleep;
use tracing::{info, error};

use crate::scraping::{kick, lib::{drop_dups, MessageOut}};


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
        let tab = match kick::go_to(browser, &streamer) {
            Ok(t) => t,
            Err(e) => {
                error!("go_to(): {}", e);
                return;
            } 
        };

        socket.join(streamer.to_owned()).unwrap();
        rooms.insert(streamer.to_owned(), 1);
        tokio::task::spawn(async move {
            let mut cache = HashSet::<String>::new();
            loop {
                // underscore to silence warning
                let mut _has_connections = false;
                {
                    let rooms = Arc::clone(&state);
                    let rooms = rooms.lock().unwrap();
                    _has_connections =  rooms.contains_key(&streamer);
                }
                if !_has_connections { 
                    tab.close_target().unwrap();
                    break; 
                }
                let (messages, new_cache) = match kick::scrape(&tab) {
                    Ok(msgs) => drop_dups(msgs, cache),
                    Err(e) => {
                        error!("scrape(): {e}");
                        break;
                    }
                };
                cache = new_cache;

                for msg in messages {
                    match socket
                        .within(streamer.to_owned())
                        .emit("chat", MessageOut::from(msg)) {
                        Ok(_) => {},
                        Err(e) => {
                            tab.close_target().unwrap();
                            error!("socket: {e}");
                            return;
                        }
                    };
                }
                sleep(Duration::from_millis(1000)).await;
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
