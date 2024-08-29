use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use headless_chrome::{Browser, Tab};
use scraper::{Html, Selector};
use tracing::info;

use crate::Result;
use crate::scraping::lib::{Message, PageMeta};

pub fn go_to(browser: Arc<Browser>, streamer: &str) -> Result<Arc<Tab>> {
    let url: String = format!("https://kick.com/{}/chatroom", streamer);
    info!("{url}");
    let tab = browser.new_tab()?;
    tab.set_default_timeout(Duration::from_secs(5));

    let page_meta = PageMeta::new(&url);
    tab.set_user_agent(page_meta.get_user_agent(), None, None)?;
    tab.set_cookies(vec![page_meta.cookie])?;
    match tab.navigate_to(&url)?.wait_for_element(".chat-entry") {
        Ok(_) => {},
        Err(e) => {
            tab.close_target().unwrap();
            return Err(crate::Error::Scraping(e));
        }
    };
    return Ok(tab);
}

pub fn scrape(tab: &Arc<Tab>) -> Result<Vec<Message>> {
    let chat = Selector::parse(".chat-entry")?;
    let username = Selector::parse(".chat-message-identity > .chat-entry-username")?;
    let chat_content = Selector::parse(".chat-entry-content")?;
    let emote_content = Selector::parse(".chat-emote-container > div > img")?;


    let mut messages: Vec<Message> = Vec::new();
    let content = Html::parse_document(tab.get_content()?.as_str());
    for el in content.select(&chat) {
        let chat_fragment = Html::parse_fragment(&el.inner_html());
        let Some(username) = chat_fragment.select(&username).next() else {
            return Err(crate::Error::Static("username not found".to_string()));
        };

        let mut content: Option<Rc<str>> = None;
        let mut emote_html: Option<Rc<str>> = None;

        if let Some(msg) = chat_fragment.select(&chat_content).next() {
            content = Some(Rc::from(msg.inner_html())); 
        };
        if let Some(emote) = chat_fragment.select(&emote_content).next() {
            emote_html = Some(
                Rc::from(emote.value().attr("src").unwrap().to_string())
            );
        };

        messages.push(Message::new(
            Rc::from(username.inner_html()),
            Rc::from(username.value().attr("style").unwrap().to_string()),
            content,
            emote_html
        ));
    }

    return Ok(messages);
}

