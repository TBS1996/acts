use crate::activity::Activity;
use crate::ActID;
use crate::Conn;
use crate::Message;
use iced::widget::{button, column, text, text_input};
use rusqlite::Connection;

use iced::widget::{Button, Column, Container, Slider};
use iced::{Alignment, Color, Element, Length, Renderer, Sandbox, Settings};

pub struct EditPage {
    pub activity: Activity,
}

impl EditPage {
    pub fn new(conn: &Conn, id: ActID) -> Self {
        Self {
            activity: Activity::fetch_activity(conn, id).unwrap(),
        }
    }
    pub fn view(&self) -> Element<'static, Message> {
        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("hey", &self.activity.text, Message::EditInputChanged)
                .padding(20)
                .size(30);
        column![
            text_input,
            button("go back to main").on_press(Message::EditGotoMain),
            button("Delete").on_press(Message::EditDeleteActivity(self.activity.id)),
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }
}
