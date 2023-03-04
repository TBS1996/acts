use crate::activity::Activity;

use crate::Conn;
use crate::Message;
use iced::widget::{column, Column};

use iced::{Alignment, Element, Sandbox};

#[derive(Debug)]
pub struct Picker {
    activities: Vec<Activity>,
}

impl Picker {
    pub fn new(conn: &Conn) -> Self {
        let activities = Activity::fetch_all_activities(conn);
        Self { activities }
    }

    pub fn view_activities(&self) -> Element<'static, Message> {
        let mut some_vec = Vec::new();
        for act in &self.activities {
            let actbutton: iced::widget::button::Button<Message> =
                iced::widget::button(iced::widget::text::Text::new(act.text))
                    .on_press(Message::PickAct(act.id));
            let row = iced::Element::new(iced::widget::row![actbutton]);
            some_vec.push(row);
        }

        let back_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Edit")).on_press(Message::GoBack);
        let text = iced::Element::new(iced::widget::text::Text::new("pick an activity!"));
        column![
            text,
            back_button,
            iced::widget::Column::with_children(some_vec),
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }
}
