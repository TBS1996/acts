use crate::ActID;

use crate::activity::Activity;

use crate::Conn;
use crate::MainMessage;
use crate::Message;
use crate::Page;


use crate::IntoMessage;

use iced::widget::Column;

use iced::{Alignment, Element};

#[derive(Debug)]
pub struct TreeView {
    conn: Conn,
}

impl Page for TreeView {
    fn view(&self) -> Element<'static, Message> {
        let some_vec = self.view_recursive();

        let back_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Go back"))
                .on_press(MainMessage::GoBack.into_message());

        iced::widget::column![back_button, Column::with_children(some_vec)]
            .padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}

impl TreeView {
    pub fn new(conn: Conn) -> Self {
        Self { conn }
    }

    fn view_recursive(&self) -> Vec<Element<'static, Message>> {
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
                    iced::widget::text::Text::new(format!("{}%", kid.assigned)),
                )
                .on_press(MainMessage::NewAssign(kid.id).into_message());

                let edit_button: iced::widget::button::Button<Message> =
                    iced::widget::button(iced::widget::text::Text::new(kid.text))
                        .on_press(MainMessage::NewEdit(kid.id).into_message());

                let parent_button: iced::widget::button::Button<Message> =
                    iced::widget::button(iced::widget::text::Text::new(":"))
                        .on_press(MainMessage::ChooseParent { child: kid.id }.into_message());

                let row = iced::Element::new(iced::widget::row![
                    parent_button,
                    padding,
                    edit_button,
                    single_pad,
                    assigned,
                ]);
                elms.push(row);
                recursive(conn, elms, Some(kid.id), depth + 1);
            }
        }

        let mut elms = vec![];
        recursive(&self.conn, &mut elms, None, 0);
        elms
    }
}
