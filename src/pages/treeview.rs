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

    fn view_recursive(
        &self,
        conn: &Conn,
        parent: Option<ActID>,
        depth: usize,
    ) -> Vec<Element<'static, Message>> {
        fn recursive(
            conn: &Conn,
            elms: &mut Vec<Element<'static, Message>>,
            parent: Option<ActID>,
            depth: usize,
        ) {
            let kids = Activity::fetch_children(conn, parent);

            for kid in kids {
                let padding = " ".repeat(depth * 6);
                let padding = iced::Element::new(iced::widget::text::Text::new(padding));

                let single_pad = iced::Element::new(iced::widget::text::Text::new(" "));

                //let elm = iced::Element::new(iced::widget::text::Text::new(kid.display(conn)));

                let assigned: iced::widget::button::Button<Message> = iced::widget::button(
                    iced::widget::text::Text::new(format!("{}%", kid.assigned.to_string())),
                )
                .on_press(Message::GoAssign(kid.id));

                let edit_button: iced::widget::button::Button<Message> =
                    iced::widget::button(iced::widget::text::Text::new(kid.text))
                        .on_press(Message::EditActivity(kid.id));

                let parent_button: iced::widget::button::Button<Message> =
                    iced::widget::button(iced::widget::text::Text::new(":"))
                        .on_press(Message::ChooseParent { child: kid.id });

                let row = iced::Element::new(iced::widget::row![
                    parent_button,
                    padding,
                    edit_button,
                    single_pad,
                    assigned
                ]);
                elms.push(row);
                recursive(conn, elms, Some(kid.id), depth + 1);
            }
        }

        let mut elms = vec![];
        recursive(conn, &mut elms, None, 0);
        elms
    }

    pub fn view_activities(&self, conn: &Conn) -> Element<'static, Message> {
        if let Some(picker) = &self.picker {
            return picker.1.view_activities();
        }

        if let Some(x) = &self.edit_assignment {
            return x.view();
        }

        let some_vec = self.view_recursive(conn, None, 0);

        let back_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Go back"))
                .on_press(Message::GoBack);

        column![back_button, Column::with_children(some_vec)]
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}
