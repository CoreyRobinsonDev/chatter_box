use std::{collections::HashSet, rc::Rc};

#[derive(Debug, serde::Serialize)]
pub struct MessageOut {
    pub username: Rc<str>,
    pub user_color: Rc<str>,
    pub content: Option<Rc<str>>,
    pub emote_html: Option<Rc<str>>
}

impl MessageOut {
    pub fn from(message: Message) -> Self {
        return Self { 
            username: message.username, 
            user_color: message.user_color, 
            content: message.content, 
            emote_html: message.emote_html 
        }
    }
}

pub struct Message {
    pub id: String,
    pub username: Rc<str>,
    pub user_color: Rc<str>,
    pub content: Option<Rc<str>>,
    pub emote_html: Option<Rc<str>>
}

impl Message {
    pub fn new(username: Rc<str>, user_color: Rc<str>, content: Option<Rc<str>>, emote_html: Option<Rc<str>>) -> Self {
        return Self { 
            id: encode(
                &username.to_string(), 
                content.as_ref(), 
                emote_html.as_ref()
            ),
            username, user_color, content, emote_html 
        }
    }
}


pub struct PageMeta {
    user_agents: Vec<String>,
    pub cookie: headless_chrome::protocol::cdp::Network::CookieParam
}

impl PageMeta {
    pub fn new(url: impl Into<String>) -> Self {
        let user_agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:128.0) Gecko/20100101 Firefox/128.0".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.6; rv:128.0) Gecko/20100101 Firefox/128.0".to_string(),
            "Mozilla/5.0 (X11; Linux i686; rv:128.0) Gecko/20100101 Firefox/128.0".to_string()
        ];
        let cookie = headless_chrome::protocol::cdp::Network::CookieParam {
            domain: Some(".kick.com".to_string()),
            expires: None,
            http_only: Some(true),
            name: "kick_session".to_string(),
            partition_key: None,
            path: Some("/".to_string()),
            priority: Some(headless_chrome::protocol::cdp::Network::CookiePriority::Medium),
            same_party: None,
            same_site: Some(headless_chrome::protocol::cdp::Network::CookieSameSite::Lax),
            secure: Some(false),
            source_port: None,
            source_scheme: None,
            url: Some(url.into()),
            value: String::from("eyJpdiI6IjRaczhCazlIbHFiNE80VlhocFpOeUE9PSIsInZhbHVlIjoiK0ZZTHFoOXRoVG9iaG9Nckl0M25wSG9Rcksrd1JCUnJmZU4xcEFobjFZeStDY2QzaGRKeUZOWnRTWnNKd0NGTlYwQjVHOHZsUzllRmh5OW1pTHNKS05nZjRoKzhiVU94bjVHZ3hiSnJWRGlDOVdxTXRBdUZhc0JESFE3V2ZROXgiLCJtYWMiOiI4ODMwZWNhNDQ1MDYxOTQ5ZTRiODE4MDI5MGFmN2IyNzMxMWU0NjEzYzQ2YTIzYzZhYTBmZGQ5NTRiN2NlMjhiIiwidGFnIjoiIn0%3D"),
        };
        Self { user_agents, cookie }
    }

    pub fn get_user_agent(&self) -> &String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..self.user_agents.len());
        return self.user_agents.get(idx).unwrap();
    }
}

fn encode(
    username: impl Into<String>, 
    content: Option<&Rc<str>>, 
    emote_html: Option<&Rc<str>>
) -> String {
    use base64::prelude::*;
    let mut id = username.into();
    id += content
        .unwrap_or(&Rc::<str>::from(""));
    id += emote_html
        .unwrap_or(&Rc::<str>::from(""));

    return BASE64_STANDARD.encode(id);
}

pub fn drop_dups(messages: Vec<Message>, cache: HashSet<String>) -> (Vec<Message>, HashSet<String>) {
    let mut new_cache = HashSet::<String>::new();
    let mut msgs = Vec::<Message>::new();

    for msg in messages {
        new_cache.insert(msg.id.clone());
        if !cache.contains(&msg.id) {
            msgs.push(msg);
        }
    } 

    return (msgs, new_cache);
}

