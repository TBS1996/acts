use iced::widget::{button, column, text_input};

use iced::widget::Column;
use iced::{Alignment, Element, Renderer, Sandbox, Settings};

pub fn main() -> iced::Result {
    std::env::set_var("RUST_BACKTRACE", "1");
    Counter::run(Settings::default())
}

mod activity;
mod history;
mod pages;
mod sql;
mod utils;

use crate::activity::Activity;
use crate::pages::editpage::EditPage;
use crate::pages::sessionpage::SessionPage;

type Conn = rusqlite::Connection;
type ActID = usize;

pub struct Counter {
    conn: Conn,
    textboxval: String,
    activities: Vec<Activity>,
    page: Page,
}

#[derive(Default)]
pub enum Page {
    #[default]
    Main,
    Edit(EditPage),
    NewSession(SessionPage),
}

impl Counter {
    fn view_helper(
        &self,
        activity: Activity,
        elms: &mut Vec<Element<'static, Message>>,
        depth: usize,
    ) {
        let padding = std::iter::repeat(' ').take(depth * 6).collect::<String>();

        let padding = iced::Element::new(iced::widget::text::Text::new(padding));

        let elm = iced::Element::new(iced::widget::text::Text::new(activity.display(&self.conn)));

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

        let button: iced::widget::button::Button<Message> =
            iced::widget::button(iced::widget::text::Text::new("@"))
                .on_press(Message::MainNewSession(activity.id));

        let row = iced::Element::new(iced::widget::row![
            padding,
            left_button,
            right_button,
            up_button,
            down_button,
            button,
            elm
        ]);
        elms.push(row);

        for activity in activity.children {
            self.view_helper(activity, elms, depth + 1);
        }
    }

    fn view_activities(&self) -> Vec<Element<'static, Message>> {
        let mut some_vec = Vec::new();
        for act in &self.activities {
            self.view_helper(act.clone(), &mut some_vec, 0);
        }
        some_vec
    }

    fn main_view(&self) -> Element<'static, Message> {
        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("hey", &self.textboxval, Message::MainInputChanged)
                .padding(20)
                .size(30);

        column![
            text_input,
            button("Add activity").on_press(Message::MainAddActivity),
            Column::with_children(self.view_activities()),
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }

    fn normalize_stuff(&mut self) {
        self.normalize_all_assignments();
        Activity::normalize_positions(&self.conn);
    }

    fn normalize_all_assignments(&mut self) {
        Activity::normalize_assignments(&self.conn, None);
        let mut f = |conn: &Conn, activity: &mut Activity| {
            Activity::normalize_assignments(conn, Some(activity.id));
        };

        let mut activities = Activity::fetch_all_activities(&self.conn);
        Activity::activity_walker_dfs(&self.conn, &mut activities, &mut f)
    }

    fn refresh(&mut self) {
        self.normalize_stuff();
        self.activities = Activity::fetch_all_activities(&self.conn);
        self.textboxval = String::new();
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    MainInputChanged(String),
    MainAddActivity,
    MainEditActivity(ActID),
    MainNewSession(ActID),
    MainGoUp(ActID),
    MainGoDown(ActID),
    MainGoLeft(ActID),
    MainGoRight(ActID),

    EditDeleteActivity(ActID),
    EditGotoMain,
    EditInputChanged(String),

    SessionInputChanged(String),
    SessionAddSession,

    AddSession {
        id: ActID,
        duration: std::time::Duration,
    },
}

impl Sandbox for Counter {
    type Message = Message;

    fn new() -> Self {
        let conn = sql::init();
        Activity::normalize_assignments(&conn, None);
        let activities = Activity::fetch_all_activities(&conn);
        Self {
            conn,
            textboxval: String::new(),
            activities,
            page: Page::default(),
        }
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) {
        match &mut self.page {
            Page::Main => match message {
                Message::MainEditActivity(id) => {
                    self.page = Page::Edit(EditPage::new(&self.conn, id))
                }
                Message::MainNewSession(id) => {
                    let newpage = SessionPage {
                        id,
                        duration: String::new(),
                    };
                    self.page = Page::NewSession(newpage);
                }
                Message::MainInputChanged(x) => self.textboxval = x,
                Message::MainAddActivity => {
                    let x: String = std::mem::take(&mut self.textboxval);
                    let activity = Activity::new(&self.conn, x);
                    sql::new_activity(&self.conn, &activity).unwrap();
                    self.activities.push(activity);
                }
                Message::MainGoUp(id) => {
                    Activity::go_up(&self.conn, id);
                    self.refresh();
                }
                Message::MainGoDown(id) => {
                    Activity::go_down(&self.conn, id);
                    self.refresh();
                }
                Message::MainGoRight(id) => {
                    Activity::go_right(&self.conn, id);
                    self.refresh();
                }
                Message::MainGoLeft(id) => {
                    Activity::go_left(&self.conn, id);
                    self.refresh();
                }
                _ => {
                    panic!("you forgot to add {:?} to this match arm", message)
                }
            },
            Page::Edit(editor) => match message {
                Message::EditDeleteActivity(id) => {
                    sql::delete_activity(&self.conn, id);
                    self.page = Page::Main;
                    self.refresh();
                }

                Message::EditGotoMain => {
                    self.page = Page::Main;
                    self.refresh();
                }
                Message::EditInputChanged(text) => {
                    editor.activity.modify_text(text, &self.conn);
                }
                _ => {
                    panic!("you forgot to add {:?} to this match arm", message)
                }
            },

            Page::NewSession(page) => match message {
                Message::SessionAddSession => {
                    if page.duration.is_empty() {
                        self.page = Page::Main;
                        self.refresh();
                        return;
                    }
                    page.new_session(&self.conn);
                    self.page = Page::Main;
                    self.refresh();
                }

                Message::SessionInputChanged(text) => {
                    if text.is_empty() {
                        page.duration = text;
                    } else if let Ok(_) = text.parse::<f64>() {
                        page.duration = text;
                    }
                }

                _ => {
                    panic!("you forgot to add {:?} to this match arm", message)
                }
            },
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.page {
            Page::Main => self.main_view(),
            Page::Edit(page) => page.view(),
            Page::NewSession(page) => page.view(),
        }
    }
}
