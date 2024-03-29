use crate::activity::Activity;
use crate::ActID;
use crate::Conn;
use crate::IntoMessage;
use crate::MainMessage;
use crate::Message;
use crate::Page;
use crate::PageMessage;
use iced::widget::{button, text_input};

use iced::{Alignment, Command, Element, Renderer};

#[derive(Debug)]
pub struct EditPage {
    pub activity: Activity,
    pub assigned: String,
    pub session_duration: String,
    conn: Conn,
}

impl Page for EditPage {
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

        let child_button = button("Add new child").on_press(
            MainMessage::PageAddActivity {
                parent: Some(self.activity.id),
            }
            .into_message(),
        );

        let view_note = button("View note").on_press(
            MainMessage::EditNote {
                id: self.activity.id,
            }
            .into_message(),
        );

        iced::widget::column![
            session_input,
            text_input,
            button("go back to main").on_press(MainMessage::GoBack.into_message()),
            button("Delete").on_press(MainMessage::DeleteActivity(self.activity.id).into_message()),
            child_button,
            view_note,
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
        let mut id = self.activity.id;

        let statement =
            "INSERT INTO history (id, duration, timestamp) VALUES (?1, ?2, ?3)".to_string();
        self.conn
            .execute(
                &statement,
                rusqlite::params![id.to_string(), duration, timestamp],
            )
            .unwrap();
    

    while let Some(parent) = Activity::get_parent(&self.conn, id){
        id = parent.id;
        self.conn
            .execute(
                &statement,
                rusqlite::params![id.to_string(), duration, timestamp],
            )
            .unwrap();

    
    }
    }

        

}
