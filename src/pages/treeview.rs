use crate::activity::Activity;

use crate::Conn;
use crate::Message;


use iced::{Alignment, Element, Sandbox};

#[derive(Debug)]
pub struct TreeView {
    activities: Vec<Activity>,
}

impl TreeView {
    pub fn new(conn: &Conn) -> Self {
        let activities = Activity::fetch_all_activities(conn);
        Self { activities }
    }

    fn view_helper(
        &self,
        conn: &Conn,
        activity: Activity,
        elms: &mut Vec<Element<'static, Message>>,
        depth: usize,
    ) {
        let padding = " ".repeat(depth * 6);

        let padding = iced::Element::new(iced::widget::text::Text::new(padding));

        let elm = iced::Element::new(iced::widget::text::Text::new(activity.display(conn)));

        let right_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new(">"))
                .on_press(Message::MainGoRight(activity.id));

        let left_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("<"))
                .on_press(Message::MainGoLeft(activity.id));

        let up_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("^"))
                .on_press(Message::MainGoUp(activity.id));

        let down_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("v"))
                .on_press(Message::MainGoDown(activity.id));

        let session_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Add session"))
                .on_press(Message::MainNewSession(activity.id));

        let edit_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Edit"))
                .on_press(Message::MainEditActivity(activity.id));
        let row = iced::Element::new(iced::widget::row![
            padding,
            left_button,
            right_button,
            up_button,
            down_button,
            session_button,
            edit_button,
            elm
        ]);
        elms.push(row);

        for activity in activity.children {
            self.view_helper(conn, activity, elms, depth + 1);
        }
    }

    pub fn view_activities(&self, conn: &Conn) -> Element<'static, Message> {
        let mut some_vec = Vec::new();
        for act in &self.activities {
            self.view_helper(conn, act.clone(), &mut some_vec, 0);
        }
        iced::widget::Column::with_children(some_vec)
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}
