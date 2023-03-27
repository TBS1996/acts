//pub mod assignments;
pub mod assignments;
pub mod editpage;
pub mod new_activity;
pub mod picker;
pub mod treeview;

use crate::IntoMessage;
use crate::MainMessage;
use iced::{Alignment, Command, Element, Renderer};

pub trait Page {
    fn view(&self) -> Element<'static, Message>;

    fn update(&mut self, _message: PageMessage) -> Command<Message> {
        panic!("");
    }
}

use std::fmt::Debug;

use crate::{ActID, Conn, Message, PageMessage};
use iced::widget::text_input;

pub struct ValueGetter {
    pub title: String,
    pub input: String,
    pub id: ActID,
    conn: Conn,
}

impl Page for ValueGetter {
    fn update(&mut self, message: PageMessage) -> Command<Message> {
        match message {
            PageMessage::ValueGetInput(s) => {
                if s.parse::<u32>().is_ok() || s.is_empty() {
                    self.input = s;
                }
            }
            PageMessage::ValueSubmit => {
                if self.input.parse::<u32>().is_ok() || self.input.is_empty() {
                    let statement = format!(
                        "UPDATE activities SET assigned = '{}' WHERE id = '{}'",
                        self.input.clone(),
                        self.id
                    );
                    self.conn.execute(&statement, []).unwrap();
                }
            }
            _ => panic!(),
        }
        Command::none()
    }

    fn view(&self) -> Element<'static, Message> {
        let back_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Go back"))
                .on_press(MainMessage::GoBack.into_message());

        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("wtf", &self.input, |s| {
                PageMessage::ValueGetInput(s).into_message()
            })
            .on_submit(PageMessage::ValueSubmit.into_message())
            .padding(20)
            .id(iced::widget::text_input::Id::unique())
            .size(30);

        let title = iced::Element::new(iced::widget::text::Text::new(self.title.clone()));

        iced::widget::column![back_button, title, text_input]
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}

impl ValueGetter {
    pub fn _new(conn: Conn, title: String, id: ActID) -> Self {
        Self {
            title,
            input: String::new(),
            id,
            conn,
        }
    }
}

impl Debug for ValueGetter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Hi")
    }
}
