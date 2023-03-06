use crate::ActID;

use crate::activity::Activity;

use crate::Conn;
use crate::Message;
use iced::widget::{column, pick_list, text_input, Column};
use iced::Renderer;

use iced::{Alignment, Element, Sandbox};

use super::picker::Picker;
use super::ValueGetter;

#[derive(Debug)]
pub struct TreeView {
    activities: Vec<Activity>,
    pub picker: Option<(ActID, Picker)>,
    pub edit_assignment: Option<ValueGetter>,
}

impl TreeView {
    pub fn new(conn: &Conn) -> Self {
        let activities = Activity::fetch_all_activities(conn);
        Self {
            activities,
            picker: None,
            edit_assignment: None,
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

        let elm = iced::Element::new(iced::widget::text::Text::new(activity.display(conn)));

        let assigned: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("assigned"))
                .on_press(Message::GoAssign(activity.id));

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

        if let Some(x) = &self.edit_assignment {
            return x.view();
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
