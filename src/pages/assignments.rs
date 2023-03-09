use crate::activity::Activity;
use crate::ActID;

use crate::Conn;
use crate::Message;
use iced::widget::text_input;
use iced::widget::{column, Column};
use iced::Renderer;

use iced::{Alignment, Element, Sandbox};

#[derive(Debug)]
pub struct Assignments {
    msg: String,
    parent: Option<ActID>,
    activities: Vec<Activity>,
    diff: i32,
    idx: usize,
}

impl Assignments {
    fn get_diff(&self) -> i32 {
        let mut tot: i32 = 0;

        for activity in &self.activities {
            tot += activity.assigned as i32;
        }
        tot - 100
    }

    pub fn new(conn: &Conn, parent: Option<ActID>) -> Self {
        let activities = Activity::fetch_children(conn, parent);

        let mut myself = Self {
            msg: String::new(),
            parent,
            activities,
            diff: 0,
            idx: 0,
        };

        let diff = myself.get_diff();
        let msg = format!("Current difference: {}", diff);
        myself.diff = diff;
        myself.msg = msg;

        myself
    }

    pub fn view_activities(&self) -> Element<'static, Message> {
        for (idx, act) in self.activities.iter().enumerate() {
            let actbutton: iced::widget::text_input::TextInput<'_, Message, Renderer> =
                text_input("Add activity", &act.text, Message::MainInputChanged)
                    .on_submit(|self: &Assignments| {
                        self.idx = idx;
                        Message::MainInputChanged
                    })
                    .padding(20)
                    .size(30);
            let row = iced::Element::new(iced::widget::row![actbutton]);
            some_vec.push(row);
        }

        let back_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Go back"))
                .on_press(Message::GoBack);
        let text = iced::Element::new(iced::widget::text::Text::new("pick an activity!"));

        /*
        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("wtf", &self.input, Message::ValueGetInput)
                .on_submit(Message::SubmitValue)
                .padding(20)
                .id(iced::widget::text_input::Id::unique())
                .size(30);
                */

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
