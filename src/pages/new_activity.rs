use iced::widget::{button, row, text_input};

use iced::widget::Column;
use iced::{executor, Alignment, Application, Command, Element, Renderer, Settings};

use crate::ActID;
use crate::IntoMessage;
use crate::MainMessage;
use crate::Message;
use crate::Page;
use crate::PageMessage;

#[derive(Debug)]
pub struct NewActivity {
    input: String,
    parent: Option<ActID>,
}

impl Page for NewActivity {
    fn update(&mut self, message: crate::PageMessage) -> Command<Message> {
        match message {
            PageMessage::InputChanged((_, s)) => {
                self.input = s;
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
            text_input("Add activity", &self.input, |x| {
                crate::PageMessage::InputChanged((0, x)).into_message()
            })
            .on_submit(
                MainMessage::AddActivity {
                    name: self.input.clone(),
                    parent: self.parent,
                }
                .into_message(),
            )
            .padding(20)
            .size(30);

        iced::widget::column![back_button, text_input]
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}

impl NewActivity {
    pub fn new(parent: Option<ActID>) -> Self {
        Self {
            input: String::default(),
            parent,
        }
    }
}
