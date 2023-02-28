use crate::activity::Activity;
use crate::ActID;
use crate::Conn;
use crate::Message;
use iced::widget::{button, column, text_input};

use iced::{Alignment, Element, Renderer, Sandbox};

#[derive(Debug)]
pub struct EditPage {
    pub activity: Activity,
    pub assigned: String,
    pub session_duration: String,
}

impl EditPage {
    pub fn new(conn: &Conn, id: ActID) -> Self {
        Self {
            activity: Activity::fetch_activity(conn, id).unwrap(),
            assigned: String::default(),
            session_duration: String::default(),
        }
    }

    fn maybe_add_session(&self) -> Message {
        if self.session_duration.parse::<f64>().is_ok() {
            return Message::EditAddSession;
        }
        Message::EditGotoMain
    }

    pub fn view(&self) -> Element<'static, Message> {
        let session_input: iced::widget::text_input::TextInput<'_, Message, Renderer> = text_input(
            "New session",
            &self.session_duration,
            Message::EditSessionInput,
        )
        .on_submit(self.maybe_add_session())
        .padding(20)
        .size(30);

        let assigned_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("Assign time", &self.assigned, Message::EditAssignInput)
                .on_submit(Message::EditAddAssign)
                .padding(20)
                .size(30);

        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("Edit name", &self.activity.text, Message::EditInputChanged)
                .on_submit(Message::EditGotoMain)
                .padding(20)
                .size(30);

        column![
            session_input,
            text_input,
            assigned_input,
            button("go back to main").on_press(Message::EditGotoMain),
            button("Delete").on_press(Message::EditDeleteActivity(self.activity.id)),
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
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
