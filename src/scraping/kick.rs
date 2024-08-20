use std::{thread::sleep, time::Duration};

use headless_chrome::{protocol::cdp::Network::{CookieParam, CookiePriority, CookieSameSite}, Browser, LaunchOptions};
use anyhow::Result;
use scraper::{Html, Selector};

const URL: &str = "https://kick.com/roshtein/chatroom";

pub fn scrape() -> Result<()> {
    let user_agents= vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:128.0) Gecko/20100101 Firefox/128.0",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 14.6; rv:128.0) Gecko/20100101 Firefox/128.0",
        "Mozilla/5.0 (X11; Linux i686; rv:128.0) Gecko/20100101 Firefox/128.0"
    ];
    let cookie = CookieParam {
        domain: Some(".kick.com".to_string()),
        expires: None,
        http_only: Some(true),
        name: "kick_session".to_string(),
        partition_key: None,
        path: Some("/".to_string()),
        priority: Some(CookiePriority::Medium),
        same_party: None,
        same_site: Some(CookieSameSite::Lax),
        secure: Some(false),
        source_port: None,
        source_scheme: None,
        url: Some(URL.to_string()),
        value: String::from("eyJpdiI6IjRaczhCazlIbHFiNE80VlhocFpOeUE9PSIsInZhbHVlIjoiK0ZZTHFoOXRoVG9iaG9Nckl0M25wSG9Rcksrd1JCUnJmZU4xcEFobjFZeStDY2QzaGRKeUZOWnRTWnNKd0NGTlYwQjVHOHZsUzllRmh5OW1pTHNKS05nZjRoKzhiVU94bjVHZ3hiSnJWRGlDOVdxTXRBdUZhc0JESFE3V2ZROXgiLCJtYWMiOiI4ODMwZWNhNDQ1MDYxOTQ5ZTRiODE4MDI5MGFmN2IyNzMxMWU0NjEzYzQ2YTIzYzZhYTBmZGQ5NTRiN2NlMjhiIiwidGFnIjoiIn0%3D"),
    };
    let chat = Selector::parse(".chat-entry").unwrap();
    let username = Selector::parse(".chat-message-identity > .chat-entry-username").unwrap();
    let chat_content = Selector::parse(".chat-entry-content").unwrap();
    let emote_content = Selector::parse(".chat-emote-container > div > img").unwrap();
    let browser = Browser::new(LaunchOptions {
        headless: true,
        ..Default::default()
    })?;
    let tab = browser.new_tab()?;

    tab.set_user_agent(user_agents[1], None, None).unwrap();
    tab.set_cookies(vec![cookie]).unwrap();
    tab.navigate_to(URL)?
        .wait_for_element(".chat-entry")?;


    let content = Html::parse_document(tab.get_content()?.as_str());
    for el in content.select(&chat) {
        println!("===");
        let chat_fragment = Html::parse_fragment(&el.inner_html());
        let username = chat_fragment.select(&username).next().unwrap();

        println!("{:?}", username.inner_html());
        println!("{:?}", username.value().attr("style").unwrap());
        if let Some(msg) = chat_fragment.select(&chat_content).next() {
            println!("{:?}", msg.inner_html());
        };
        if let Some(emote) = chat_fragment.select(&emote_content).next() {
            println!("{:?}", emote.value().attr("src").unwrap());
        };
    }


    return Ok(());
}

