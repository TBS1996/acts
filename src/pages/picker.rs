use crate::activity::Activity;

use crate::ActID;
use crate::Conn;
use crate::MainMessage;
use crate::Message;
use crate::PageMessage;
use iced::widget::{column, Column};

use iced::{Alignment, Element, Sandbox};

use super::Page;

#[derive(Debug)]
pub struct Picker {
    activities: Vec<Activity>,
    child: ActID,
    conn: Conn,
}

impl Page for Picker {
    fn refresh(&mut self) {}

    fn update(&mut self, message: PageMessage) -> iced::Command<Message> {
        panic!()
    }

    fn view(&self) -> Element<'static, Message> {
        let pick_parent = |parent: Option<ActID>| {
            Activity::set_parent(&self.conn, self.child, parent);
            MainMessage::GoBack.into_message()
        };
        let mut some_vec = Vec::new();
        let root_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Root")).on_press(pick_parent(None));

        for act in &self.activities {
            let actbutton: iced::widget::button::Button<Message> =
                iced::widget::button(iced::widget::text::Text::new(act.text.clone()))
                    .on_press(pick_parent(Some(act.id)));
            let row = iced::Element::new(iced::widget::row![actbutton]);
            some_vec.push(row);
        }

        let back_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Go back"))
                .on_press(MainMessage::GoBack.into_message());
        let text = iced::Element::new(iced::widget::text::Text::new("pick an activity!"));

        column![
            text,
            back_button,
            root_button,
            iced::widget::Column::with_children(some_vec),
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }
}

impl Picker {
    pub fn new(conn: Conn, child: ActID) -> Self {
        let activities = Activity::fetch_all_activities_flat(&conn);
        Self {
            activities,
            child,
            conn,
        }
    }
}
