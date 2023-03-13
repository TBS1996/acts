use crate::activity::Activity;
use crate::ActID;
use crate::PageMessage;

use crate::sql;
use crate::Conn;
use crate::MainMessage;
use crate::Message;
use iced::widget::text_input;
use iced::widget::{column, Column};
use iced::Renderer;

use iced::{Alignment, Command, Element, Sandbox};

use super::Page;

#[derive(Debug)]
pub struct Assignments {
    msg: String,
    activities: Vec<Activity>,
    conn: Conn,
}

impl Page for Assignments {
    fn refresh(&mut self) {}

    fn update(&mut self, message: crate::PageMessage) -> iced::Command<Message> {
        match message {
            PageMessage::InputChanged((idx, s)) => {
                let num = if s.is_empty() {
                    0
                } else if let Ok(num) = s.parse::<u32>() {
                    num
                } else {
                    return Command::none();
                };
                self.activities[idx].assigned = num;
            }
            PageMessage::Adjust => {
                let invec = self
                    .activities
                    .clone()
                    .iter()
                    .map(|act| act.assigned as i32)
                    .collect();
                let normalized_vec = crate::utils::normalize_vec(invec, 100);
                for (idx, x) in normalized_vec.iter().enumerate() {
                    self.activities[idx].assigned = *x as u32;
                }
            }
            _ => return Command::none(),
        };
        Command::none()
    }

    fn view(&self) -> Element<'static, Message> {
        let title = iced::Element::new(iced::widget::text::Text::new("Make diff 0 to submit"));
        let diff = {
            let diff = self.get_diff();
            let diff = format!("Current difference: {}", diff);
            iced::Element::new(iced::widget::text::Text::new(diff))
        };
        let mut some_vec = vec![];
        for (idx, act) in self.activities.iter().enumerate() {
            let assigned_button: iced::widget::text_input::TextInput<'_, Message, Renderer> =
                text_input("", &act.assigned.to_string(), move |s| {
                    PageMessage::InputChanged((idx, s)).into_message()
                })
                .padding(20)
                .width(200)
                .size(30);
            let desc = iced::Element::new(iced::widget::text::Text::new(act.text.clone()));
            let row = iced::Element::new(iced::widget::row![assigned_button, desc]);
            some_vec.push(row);
        }

        let auto_adjust: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Auto adjust"))
                .on_press(PageMessage::Adjust.into_message());

        let submit_button: iced::widget::button::Button<Message> = iced::widget::button(
            iced::widget::text::Text::new("Submit"),
        )
        .on_press(if self.get_diff() == 0 {
            for act in self.activities.iter() {
                sql::set_assigned(&self.conn, act.id, act.assigned);
            }
            MainMessage::GoBack.into_message()
        } else {
            MainMessage::NoOp.into_message()
        });

        let back_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Go back"))
                .on_press(MainMessage::GoBack.into_message());

        column![
            title,
            diff,
            iced::widget::row![auto_adjust, submit_button],
            iced::widget::Column::with_children(some_vec),
            back_button,
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }
}

impl Assignments {
    fn get_diff(&self) -> i32 {
        let mut tot: i32 = 0;

        for activity in &self.activities {
            tot += activity.assigned as i32;
        }
        tot - 100
    }

    fn update_msg(&mut self) {
        let diff = self.get_diff();
        self.msg = format!("Current difference: {}", diff);
    }

    pub fn new(conn: Conn, parent: Option<ActID>) -> Self {
        let activities = Activity::fetch_children(&conn, parent);

        let mut myself = Self {
            msg: String::new(),
            activities,
            conn,
        };

        let diff = myself.get_diff();
        let msg = format!("Current difference: {}", diff);
        myself.msg = msg;

        myself
    }
}
