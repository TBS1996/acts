use crate::ActID;

use crate::activity::Activity;

use crate::Conn;
use crate::MainMessage;
use crate::Message;
use crate::Page;
use crate::PageMessage;

use crate::IntoMessage;

use iced::widget::Column;

use iced::{Alignment, Element};

#[derive(Debug)]
pub struct Picker {
    child: ActID,
    conn: Conn,
}

impl Page for Picker {
    fn refresh(&mut self) {}
    fn update(&mut self, _message: PageMessage) -> iced::Command<Message> {
        todo!()
    }
    fn view(&self) -> Element<'static, Message> {
        let some_vec = self.view_recursive();

        let prelude = iced::Element::new(iced::widget::text::Text::new(format!(
            "Choose parent for {}",
            Activity::fetch_activity(&self.conn, self.child)
                .unwrap()
                .text
        )));
        let back_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Go back"))
                .on_press(MainMessage::GoBack.into_message());

        let root_button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("Make root node")).on_press({
                Activity::set_parent(&self.conn, self.child, None);
                MainMessage::GoBack.into_message()
            });

        iced::widget::column![
            prelude,
            root_button,
            Column::with_children(some_vec),
            back_button
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }
}

impl Picker {
    pub fn new(conn: Conn, child: ActID) -> Self {
        Self { conn, child }
    }

    fn view_recursive(&self) -> Vec<Element<'static, Message>> {
        fn recursive(
            conn: &Conn,
            elms: &mut Vec<Element<'static, Message>>,
            parent: Option<ActID>,
            depth: usize,
            chosen_kid: ActID,
        ) {
            let kids = Activity::fetch_children(conn, parent);

            for kid in kids {
                let padding = " ".repeat(depth * 6);
                let padding = iced::Element::new(iced::widget::text::Text::new(padding));

                let single_pad = iced::Element::new(iced::widget::text::Text::new(" "));
                let single_pad2 = iced::Element::new(iced::widget::text::Text::new(" "));

                let edit_button: iced::widget::button::Button<Message> =
                    iced::widget::button(iced::widget::text::Text::new(kid.text)).on_press({
                        Activity::set_parent(conn, chosen_kid, Some(kid.id));
                        MainMessage::GoBack.into_message()
                    });

                let row = iced::Element::new(iced::widget::row![
                    single_pad,
                    padding,
                    edit_button,
                    single_pad2,
                ]);
                elms.push(row);
                recursive(conn, elms, Some(kid.id), depth + 1, chosen_kid);
            }
        }

        let mut elms = vec![];
        recursive(&self.conn, &mut elms, None, 0, self.child);
        elms
    }
}
