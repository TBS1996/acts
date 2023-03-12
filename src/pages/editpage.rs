use crate::activity::Activity;
use crate::ActID;
use crate::Conn;
use crate::MainMessage;
use crate::Message;
use crate::Page;
use crate::PageMessage;
use iced::widget::{button, column, text_input};

use iced::{Alignment, Command, Element, Renderer, Sandbox};

#[derive(Debug)]
pub struct EditPage<'a> {
    pub activity: Activity,
    pub assigned: String,
    pub session_duration: String,
    conn: &'a Conn,
}

impl<'a> Page for EditPage<'a> {
    fn refresh(&mut self) {
        //*self = Self::new(conn, self.activity.id)
    }

    fn view(&self) -> Element<'static, Message> {
        /*
        let session_input: iced::widget::text_input::TextInput<'_, Message, Renderer> = text_input(
            "New session",
            &self.session_duration,
            PageMessage::InputChanged,
        )
        .on_submit(self.maybe_add_session())
        .padding(20)
        .size(30);

        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("Edit name", &self.activity.text, Message::EditInputChanged)
                .on_submit(Message::GoBack)
                .padding(20)
                .size(30);
                */

        column![
            //   session_input,
            //    text_input,
            button("go back to main").on_press(MainMessage::GoBack.into_message()),
            button("Delete").on_press(MainMessage::DeleteActivity(self.activity.id).into_message()),
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }

    fn update(&mut self, message: PageMessage) -> iced::Command<Message> {
        match message {
            _ => Command::none(),
        }
    }
}

impl<'a> EditPage<'a> {
    pub fn new(conn: &'a Conn, id: ActID) -> Self {
        Self {
            activity: Activity::fetch_activity(conn, id).unwrap(),
            assigned: String::default(),
            session_duration: String::default(),
            conn,
        }
    }

    fn maybe_add_session(&self) -> Message {
        if self.session_duration.parse::<f64>().is_ok() {
            return Message::Todo;
        }
        Message::Todo
    }

    pub fn new_session(&self, conn: &Conn) {
        let timestamp = crate::utils::current_unix().as_secs();
        let duration = self.session_duration.parse::<f64>().unwrap();
        let statement =
            "INSERT INTO history (id, duration, timestamp) VALUES (?1, ?2, ?3)".to_string();
        conn.execute(
            &statement,
            rusqlite::params![self.activity.id, duration, timestamp],
        )
        .unwrap();
    }
}
