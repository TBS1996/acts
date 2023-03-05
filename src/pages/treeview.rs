use crate::ActID;

use crate::activity::Activity;

use crate::Conn;
use crate::Message;
use iced::widget::{column, pick_list, text_input, Column};
use iced::Renderer;

use iced::{Alignment, Element, Sandbox};

use super::picker::Picker;

#[derive(Debug)]
pub struct TreeView {
    activities: Vec<Activity>,
    // ActID is the card that will be moved to below the parent chosen
    pub picker: Option<(ActID, Picker)>,
}

impl TreeView {
    pub fn new(conn: &Conn) -> Self {
        let activities = Activity::fetch_all_activities(conn);
        Self {
            activities,
            picker: None,
        }
    }

    pub fn refresh(&mut self, conn: &Conn) {
        *self = Self::new(conn)
    }

    fn view_helper(
        &self,
        conn: &Conn,
        activity: Activity,
        elms: &mut Vec<Element<'static, Message>>,
        depth: usize,
    ) {
        let padding = ">".repeat(depth * 6);
        let padding = iced::Element::new(iced::widget::text::Text::new(padding));

        let assigned: iced::widget::text_input::TextInput<'_, Message, Renderer> = text_input(
            "hey",
            &activity.assigned.to_string(),
            Message::MainInputChanged,
        )
        .on_submit(Message::MainAddActivity)
        .padding(10)
        .width(75)
        .size(20);

        let elm = iced::Element::new(iced::widget::text::Text::new(activity.display(conn)));

        let edit_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Edit"))
                .on_press(Message::MainEditActivity(activity.id));

        let parent_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new(":"))
                .on_press(Message::ChooseParent { child: activity.id });

        let row = iced::Element::new(iced::widget::row![
            padding,
            parent_button,
            edit_button,
            elm,
            assigned
        ]);
        elms.push(row);

        for activity in activity.children {
            self.view_helper(conn, activity, elms, depth + 1);
        }
    }

    pub fn view_activities(&self, conn: &Conn) -> Element<'static, Message> {
        if let Some(picker) = &self.picker {
            return picker.1.view_activities();
        }

        let mut some_vec = Vec::new();
        for act in &self.activities {
            self.view_helper(conn, act.clone(), &mut some_vec, 0);
        }
        let back_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Go back"))
                .on_press(Message::GoBack);

        column![back_button, Column::with_children(some_vec)]
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}
