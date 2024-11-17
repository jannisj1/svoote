use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smartstring::{Compact, SmartString};

#[derive(Deserialize, Serialize)]
pub struct WSMessage {
    pub cmd: SmartString<Compact>,
    pub data: Value,
}

impl Into<Message> for WSMessage {
    fn into(self) -> Message {
        return Message::Text(serde_json::to_string(&self).unwrap());
    }
}

impl WSMessage {
    pub fn parse(message: Message) -> Option<Self> {
        if let Ok(text) = message.into_text() {
            if let Ok(msg) = serde_json::from_str::<WSMessage>(&text) {
                return Some(msg);
            }
        }

        return None;
    }
}
