//pub mod assignments;
pub mod editpage;
pub mod picker;
pub mod treeview;

use iced::{Alignment, Command, Element, Renderer, Sandbox, Settings};

pub trait Page {
    fn refresh(&mut self);

    fn view(&self) -> Element<'static, Message>;

    fn update(&mut self, message: PageMessage) -> Command<Message>;
}

use std::fmt::Debug;

use crate::pages::treeview::TreeView;
use crate::{ActID, Conn, Message, PageMessage};
use iced::widget::{button, column, pick_list, row, text, text_input};

use iced::widget::Column;

pub struct ValueGetter {
    pub title: String,
    pub input: String,
    pub id: ActID,
}

impl ValueGetter {
    pub fn new(title: String, id: ActID) -> Self {
        Self {
            title,
            input: String::new(),
            id,
        }
    }

    pub fn view(&self) -> Element<'static, Message> {
        let back_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Go back"))
                .on_press(Message::GoBack);

        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("wtf", &self.input, Message::ValueGetInput)
                .on_submit(Message::SubmitValue)
                .padding(20)
                .id(iced::widget::text_input::Id::unique())
                .size(30);

        let title = iced::Element::new(iced::widget::text::Text::new(self.title.clone()));

        column![back_button, title, text_input]
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}

impl Debug for ValueGetter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Hi")
    }
}
