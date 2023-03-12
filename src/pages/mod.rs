//pub mod assignments;
pub mod editpage;
pub mod picker;
pub mod treeview;

use iced::{Alignment, Command, Element, Renderer, Sandbox, Settings};
// PoC
pub trait Page {
    fn refresh(&mut self);

    fn view(&self) -> Element<'static, Message> {
        if let Some(page) = self.get_subpage() {
            page.view()
        } else {
            self.view_self()
        }
    }

    fn view_self(&self) -> Element<'static, Message>;

    fn clear_subpage(&mut self);

    fn get_subpage(&self) -> Option<&Self>;

    fn get_subpage_mut(&mut self) -> Option<&mut Self>;

    fn update(&mut self, message: Message) -> Command<Message> {
        if let Some(page) = self.get_subpage_mut() {
            page.update(message)
        } else {
            self.update_self(message)
        }
    }

    fn update_self(&mut self, message: Message) -> Command<Message>;
}

use std::fmt::Debug;

use crate::pages::treeview::TreeView;
use crate::{ActID, Conn, Message};
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
