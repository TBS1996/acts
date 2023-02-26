use crate::ActID;
use crate::Conn;
use crate::Message;
use iced::widget::{button, column, text_input};

use iced::{Alignment, Element, Renderer, Sandbox};

#[derive(Default)]
pub struct SessionPage {
    pub id: ActID,
    pub duration: String,
}

impl SessionPage {
    pub fn view(&self) -> Element<'static, Message> {
        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("hey", &self.duration, Message::SessionInputChanged)
                .on_submit(Message::SessionAddSession)
                .padding(20)
                .size(30);
        column![text_input,]
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }

    pub fn new_session(&self, conn: &Conn) {
        let timestamp = crate::utils::current_unix().as_secs();
        let duration = self.duration.parse::<f64>().unwrap();
        let statement =
            format!("INSERT INTO history (id, duration, timestamp) VALUES (?1, ?2, ?3)");
        conn.execute(&statement, rusqlite::params![self.id, duration, timestamp])
            .unwrap();
    }
}
