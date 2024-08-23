use std::sync::Arc;

use headless_chrome::{Browser, Tab};
use scraper::{Html, Selector};
use tracing::info;

use crate::Result;
use crate::scraping::lib::{Message, PageMeta};

pub fn go_to(browser: Arc<Browser>, streamer: &str) -> Result<Arc<Tab>> {
    let url: String = format!("https://kick.com/{}/chatroom", streamer);
    info!("{url}");
    let tab = browser.new_tab()?;

    let page_meta = PageMeta::new(&url);
    tab.set_user_agent(page_meta.get_user_agent(), None, None)?;
    tab.set_cookies(vec![page_meta.cookie])?;
    tab.navigate_to(&url)?.wait_for_element(".chat-entry")?;

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
        let username = chat_fragment.select(&username).next().unwrap();

        let mut message = Message {
            username: username.inner_html(),
            user_color: username.value().attr("style").unwrap().to_string(),
            content: None,
            emote_html: None
        };
        if let Some(msg) = chat_fragment.select(&chat_content).next() {
            message.content = Some(msg.inner_html()); 
        };
        if let Some(emote) = chat_fragment.select(&emote_content).next() {
            message.emote_html = Some(
                emote.value().attr("src").unwrap().to_string()
            );
        };
        messages.push(message);
    }

    return Ok(messages);
}

