use crate::pages::treeview::TreeView;
use iced::widget::{button, row};
use pages::assignments::Assignments;
use std::rc::Rc;

use crate::pages::new_activity::NewActivity;

use iced::widget::Column;
use iced::{executor, Alignment, Application, Command, Element, Settings};
use pages::picker::Picker;

pub fn main() -> iced::Result {
    std::env::set_var("RUST_BACKTRACE", "1");

    /*
    let _guard =  sentry::init((
        "https://54319a6197f6416598c508efdd682c0a:f721ba6b7dbe49359e016a2104953411@o4504644012736512.ingest.sentry.io/4504751752937472",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 1.0,
            enable_profiling: true,
            profiles_sample_rate: 1.0,
            ..Default::default()
        },
    ));
    */

    App::run(Settings::default())
}

mod activity;
mod history;
mod pages;
mod sql;
mod utils;

use crate::activity::Activity;
use crate::pages::editpage::EditPage;
use crate::pages::Page;

type Conn = Rc<rusqlite::Connection>;
type ActID = usize;

pub struct App {
    conn: Conn,
    textboxval: String,
    pages: Vec<Box<dyn Page>>,
}

impl App {
    fn view_activities(&self) -> Vec<Element<'static, Message>> {
        let acts = Self::view_by_priority(self);

        let mut wtf = vec![];

        for act in acts {
            let button: iced::widget::button::Button<Message> =
                iced::widget::button(iced::widget::text::Text::new(act.display_flat(&self.conn)))
                    .on_press(Message::MainMessage(MainMessage::NewEdit(act.id)));
            let row = iced::Element::new(iced::widget::row![button]);
            wtf.push(row);
        }
        wtf
    }

    fn view_by_priority(&self) -> Vec<Activity> {
        let mut activities = Activity::fetch_all_activities_flat(&self.conn)
            .into_iter()
            .filter(|act| Activity::fetch_children(&self.conn, Some(act.id)).is_empty())
            .collect();
        crate::Activity::assign_priorities(&self.conn, &mut activities);

        fn recursive(leaves: &mut Vec<Activity>, activity: &mut Activity) {
            if activity.children.is_empty() {
                leaves.push(activity.clone());
            } else {
                for child in activity.children.iter_mut() {
                    recursive(leaves, child);
                }
            }
        }
        let mut leaves = vec![];

        for activity in activities.iter_mut() {
            if activity.children.is_empty() {
                leaves.push(activity.clone());
            } else {
                for child in activity.children.iter_mut() {
                    recursive(&mut leaves, child);
                }
            }
        }

        leaves.sort_by_key(|leaf| std::cmp::Reverse((leaf.priority * 1000.) as u64));
        leaves
    }

    fn main_view(&self) -> Element<'static, Message> {
        let new_activity_button = button("Add activity")
            .on_press(MainMessage::PageAddActivity { parent: None }.into_message());
        let refresh_button = button("Refresh").on_press(MainMessage::Refresh.into_message());
        let treeview_button = button("view tree").on_press(MainMessage::NewTreeView.into_message());

        iced::widget::column![
            row![new_activity_button, refresh_button, treeview_button],
            Column::with_children(self.view_activities())
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }

    fn refresh(&mut self) {
        Activity::normalize_assignments(&self.conn);
    }
}

/// Messages that are handled in main.rs
#[derive(Debug, Clone)]
pub enum MainMessage {
    GoBack,
    Refresh,
    DeleteActivity(ActID),
    AddActivity { name: String, parent: Option<ActID> },
    PageAddActivity { parent: Option<ActID> },
    InputChanged(String),
    NewTreeView,
    NewAssign(ActID),
    NewEdit(ActID),
    ChooseParent { child: ActID },
    SetParent { child: ActID, parent: Option<ActID> },
    NoOp,
}

/// Messages that are handled in the last page of the pages-vector.
/// It's fine for the same variant to be used by different pages for
/// different things wherever the name and signature makes sense.
#[derive(Debug, Clone)]
pub enum PageMessage {
    // if there are multiple text_inputs, usize will differentiate them
    // so you know which one to update.
    InputChanged((usize, String)),
    PickAct(Option<ActID>),
    ValueSubmit,
    ValueGetInput(String),
    Adjust,
}

pub trait IntoMessage {
    fn into_message(self) -> Message;
}

impl IntoMessage for MainMessage {
    fn into_message(self) -> Message {
        Message::MainMessage(self)
    }
}

impl IntoMessage for PageMessage {
    fn into_message(self) -> Message {
        Message::PageMessage(self)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    MainMessage(MainMessage),
    PageMessage(PageMessage),
    InputChanged(String),
    Todo,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let conn = sql::init();
        let app = Self {
            conn,
            textboxval: String::new(),
            pages: vec![],
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        self.refresh();
        match message {
            Message::MainMessage(mainmsg) => match mainmsg {
                MainMessage::PageAddActivity { parent } => {
                    let x = Box::new(NewActivity::new(parent));
                    self.pages.push(x);
                }
                MainMessage::GoBack => {
                    self.pages.pop();
                }
                MainMessage::NewEdit(id) => {
                    let x = Box::new(EditPage::new(self.conn.clone(), id));
                    self.pages.push(x);
                }

                MainMessage::SetParent { child, parent } => {
                    // safe unwrap as || lazily evaluates from left to right.
                    if parent.is_none() || parent.unwrap() != child {
                        Activity::set_parent(&self.conn, child, parent);
                    }
                    self.pages.pop();
                }

                MainMessage::NewAssign(id) => {
                    let parent = Activity::get_parent(&self.conn, id).map(|act| act.id);
                    let x = Box::new(Assignments::new(self.conn.clone(), parent));
                    self.pages.push(x);
                }
                MainMessage::NewTreeView => {
                    let x = Box::new(TreeView::new(self.conn.clone()));
                    self.pages.push(x);
                }
                MainMessage::Refresh => self.refresh(),
                MainMessage::DeleteActivity(id) => {
                    Activity::delete_activity(&self.conn, id);
                    self.pages.pop();
                }
                MainMessage::AddActivity { name, parent } => {
                    let activity = Activity::new(&self.conn, name, parent);
                    sql::new_activity(&self.conn, &activity).unwrap();
                    self.pages.pop();
                }
                MainMessage::InputChanged(val) => {
                    self.textboxval = val;
                }
                MainMessage::ChooseParent { child } => {
                    self.pages
                        .push(Box::new(Picker::new(self.conn.clone(), child)));
                }
                MainMessage::NoOp => {}
            },
            Message::PageMessage(pagemsg) => {
                if let Some(page) = self.pages.last_mut() {
                    return page.update(pagemsg);
                } else {
                    panic!("ey");
                }
            }
            Message::Todo => panic!(),
            _ => panic!(),
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        if let Some(page) = self.pages.last() {
            page.view()
        } else {
            self.main_view()
        }
    }
}

pub fn matches_100(vec: &Vec<u32>) -> bool {
    let mut tot = 0;
    for num in vec {
        tot += num;
    }
    tot == 100
}
