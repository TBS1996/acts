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
pub struct EditPage {
    pub activity: Activity,
    pub assigned: String,
    pub session_duration: String,
    conn: Conn,
}

impl Page for EditPage {
    fn refresh(&mut self) {
        //*self = Self::new(conn, self.activity.id)
    }

    fn view(&self) -> Element<'static, Message> {
        let session_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("New session", &self.session_duration, |s| {
                PageMessage::InputChanged((0, s)).into_message()
            })
            .on_submit(self.maybe_add_session())
            .padding(20)
            .size(30);

        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("Edit name", &self.activity.text, |s| {
                PageMessage::InputChanged((1, s)).into_message()
            })
            .on_submit(MainMessage::GoBack.into_message())
            .padding(20)
            .size(30);

        column![
            session_input,
            text_input,
            button("go back to main").on_press(MainMessage::GoBack.into_message()),
            button("Delete").on_press(MainMessage::DeleteActivity(self.activity.id).into_message()),
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }

    fn update(&mut self, message: PageMessage) -> iced::Command<Message> {
        match message {
            PageMessage::InputChanged((0, s)) => {
                if s.is_empty() || s.parse::<u32>().is_ok() {
                    self.session_duration = s;
                }
            }
            PageMessage::InputChanged((1, s)) => {
                self.activity.modify_text(s, &self.conn);
            }
            _ => {}
        };
        Command::none()
    }
}

impl EditPage {
    pub fn new(conn: Conn, id: ActID) -> Self {
        Self {
            activity: Activity::fetch_activity(&conn, id).unwrap(),
            assigned: String::default(),
            session_duration: String::default(),
            conn,
        }
    }

    fn maybe_add_session(&self) -> Message {
        if self.session_duration.parse::<f64>().is_ok() {
            self.new_session();
        }
        MainMessage::GoBack.into_message()
    }

    pub fn new_session(&self) {
        let timestamp = crate::utils::current_unix().as_secs();
        let duration = self.session_duration.parse::<f64>().unwrap();
        let statement =
            "INSERT INTO history (id, duration, timestamp) VALUES (?1, ?2, ?3)".to_string();
        self.conn
            .execute(
                &statement,
                rusqlite::params![self.activity.id, duration, timestamp],
            )
            .unwrap();
    }
}
