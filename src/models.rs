
#[derive(serde::Serialize)]
pub struct MessageOut {
    pub text: String,
    pub user: String,
    pub date: chrono::DateTime<chrono::Utc>
}

#[derive(Debug)]
pub struct Chatroom {
    pub name: String,
    pub listeners: u8
}

impl Chatroom {
    pub fn new(name: String) -> Self {
        return Self { name, listeners: 1 };
    }
}

#[derive(Debug)]
pub struct Server {
    pub chatrooms: Vec<Chatroom>
}

impl Server {
    pub fn new() -> Self {
        return Self { chatrooms: Vec::new() };
    }

    pub fn contains(&self, chatroom: impl Into<String>) -> bool {
        let mut bool = false;
        let chatroom = chatroom.into();

        for room in self.chatrooms.iter() {
            if room.name == chatroom {
                bool = true;
                break;
            }
        }

        return bool;
    }

    pub fn remove(&mut self, chatroom: impl Into<String>) {
        let mut idx: Option<usize> = None;
        let chatroom = chatroom.into();
        for (i, room) in self.chatrooms.iter().enumerate() {
            if room.name == chatroom {
                idx = Some(i);
            }
        }

        if idx.is_some() {
            self.chatrooms.remove(idx.unwrap());
        }
    }

    pub fn find(&mut self, chatroom: impl Into<String>) -> Option<&mut Chatroom> {
        let chatroom = chatroom.into();
        for room in self.chatrooms.iter_mut() {
            if room.name == chatroom {
                return Some(room);
            }
        }

        return None;
    }
}


