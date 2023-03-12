use crate::pages::treeview::TreeView;
use iced::widget::{button, column, pick_list, row, text_input};
use std::rc::Rc;

use iced::widget::Column;
use iced::{executor, Alignment, Application, Command, Element, Renderer, Sandbox, Settings};
use pages::picker::Picker;
use pages::ValueGetter;

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
    activities: Vec<Activity>,
    page: Vec<Box<dyn Page>>,
}

impl App {
    fn last_page(&mut self) -> Option<&mut Box<dyn Page>> {
        self.page.last_mut()
    }

    fn view_activities(&self) -> Vec<Element<'static, Message>> {
        let acts = Self::view_by_priority(&self);

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
        /*
        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> = text_input(
            "Add activity",
            &self.textboxval,
            Message::MainMessage(MainMessage::InputChanged),
        )
        .on_submit(MainMessage::AddActivity.into_message())
        .padding(20)
        .size(30);

        */
        let refresh_button = button("Refresh").on_press(MainMessage::Refresh.into_message());
        let treeview_button = button("view tree").on_press(MainMessage::NewTreeView.into_message());

        column![
            //   text_input,
            row![refresh_button, treeview_button],
            Column::with_children(self.view_activities())
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }

    fn refresh(&mut self) {
        self.activities = Activity::fetch_all_activities(&self.conn);
        self.textboxval = String::new();
        Activity::normalize_assignments(&self.conn);
        for page in self.page.iter_mut() {
            page.refresh();
        }
    }
}

/// Messages that are handled in main.rs
#[derive(Debug, Clone)]
pub enum MainMessage {
    GoBack,
    Refresh,
    DeleteActivity(ActID),
    AddActivity,
    InputChanged(String),
    NewTreeView,
    NewAssign(ActID),
    NewEdit(ActID),
}

impl MainMessage {
    pub fn into_message(self) -> Message {
        Message::MainMessage(self)
    }
}

/// Messages that are handled in the last page of the pages-vector.
#[derive(Debug, Clone)]
pub enum PageMessage {
    InputChanged(String),
    PickAct(Option<ActID>),
    ValueSubmit,
    ValueGetInput(String),
    ChooseParent { child: ActID },
}

impl PageMessage {
    pub fn into_message(self) -> Message {
        Message::PageMessage(self)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    MainMessage(MainMessage),
    PageMessage(PageMessage),
    InputChanged(String),
    Todo,
    /*
    MainAssigned(String),
    MainNewSession(ActID),
    MainRefresh,

    EditGotoMain,
    EditInputChanged(String),
    EditAssignInput(String),
    EditSessionInput(String),
    EditAddSession,
    EditAddAssign,

    GoToTree,
    PickAct(Option<ActID>),
    ChooseParent { child: ActID },

    GoBack,
    SubmitValue,
    ValueGetInput(String),
    GoAssign(ActID),
    DeleteActivity,
    InputChangeIndexed(usize, String),
    */
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let conn = sql::init();
        let activities = Activity::fetch_all_activities(&conn);
        let app = Self {
            conn,
            textboxval: String::new(),
            activities,
            page: vec![],
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::MainMessage(mainmsg) => match mainmsg {
                MainMessage::GoBack => {
                    self.page.pop();
                }
                MainMessage::NewEdit(id) => {
                    let x = Box::new(EditPage::new(&self.conn, id));
                    self.page.push(x);
                }

                MainMessage::NewAssign(id) => {
                    let x = Box::new(ValueGetter::new("assign some stuff".to_string(), id));
                    self.page.push(x);
                }
                MainMessage::NewTreeView => {
                    let x = Box::new(TreeView::new(self.conn.clone()));
                    self.page.push(x);
                }
                MainMessage::Refresh => self.refresh(),
                MainMessage::DeleteActivity(id) => {
                    Activity::delete_activity(&self.conn, id);
                    self.refresh();
                }
                MainMessage::AddActivity => {
                    let x: String = std::mem::take(&mut self.textboxval);
                    let activity = Activity::new(&self.conn, x);
                    sql::new_activity(&self.conn, &activity).unwrap();
                    self.activities.push(activity);
                    self.refresh();
                }
                MainMessage::InputChanged(val) => {
                    self.textboxval = val;
                }
            },
            Message::PageMessage(pagemsg) => {
                if let Some(page) = self.page.last_mut() {
                    return page.update(pagemsg);
                }
            }
            Message::Todo => panic!(),
            _ => panic!(),
        }

        Command::none()
    }

    /*

    if let Some(page) = self.page.last_mut() {
        page.update(message)
    } else {
        match &message {
            Message::GoBack => {
                self.page.pop();
                Command::none()
            }
            Message::MainAddActivity => {
                let x: String = std::mem::take(&mut self.textboxval);
                let activity = Activity::new(&self.conn, x);
                sql::new_activity(&self.conn, &activity).unwrap();
                self.activities.push(activity);
                self.refresh();
                Command::none()
            }

            Message::EditActivity(id) => {
                let x = Box::new(EditPage::new(&self.conn, *id));
                self.page.push(x);
                Command::none()
            }

            _ => Command::none(),
        }
        */

    /*


    match &message {
            Message::EditActivity(id) => {
                self.page = Page::Edit(EditPage::new(&self.conn, id));
            }
            Message::MainInputChanged(x) => {
                self.textboxval = x;
            }

            Message::MainRefresh => {
                self.refresh();
            }
            Message::GoToTree => {
                self.page = Page::TreeView(TreeView::new(&self.conn));
            }

            Message::EditDeleteActivity(id) => {
                Activity::delete_activity(&self.conn, id);
                self.page = Page::Main;
                self.refresh();
            }

            Message::EditAddSession => {
                editor.new_session(&self.conn);
                self.page = Page::Main;
                self.refresh();
            }

            Message::EditAssignInput(text) => {
                if text.is_empty() || text.parse::<f64>().is_ok() {
                    editor.assigned = text;
                }
            }
            Message::EditAddAssign => {
                if let Ok(num) = editor.assigned.parse::<u32>() {
                    let statement = format!(
                        "UPDATE activities SET assigned = {} WHERE id = {}",
                        num, editor.activity.id
                    );
                    self.conn.execute(&statement, []).unwrap();
                }
            }

            Message::EditGotoMain => {
                self.page = Page::Main;
                self.refresh();
            }

            Message::EditInputChanged(text) => {
                editor.activity.modify_text(text, &self.conn);
            }
            Message::EditSessionInput(text) => {
                if text.is_empty() || text.parse::<f64>().is_ok() {
                    editor.session_duration = text;
                }
            }
            Message::PickAct(id) => {
                Activity::set_parent(&self.conn, x.0, id);
                tree.picker = None;
                self.refresh();
            }
            Message::ChooseParent { child } => {
                tree.picker = Some((child, Picker::new(&self.conn)));
            }
            Message::GoAssign(id) => {
                let x = ValueGetter::new("assign some stuff".to_string(), id);
                tree.edit_assignment = Some(x);
            }
            Message::ValueGetInput(val) => {
                if val.is_empty() || val.parse::<u32>().is_ok() {
                    if let Some(x) = tree.edit_assignment.as_mut() {
                        x.input = val;
                    }
                }
            }
            Message::SubmitValue) => {
                if let Some(x) = tree.edit_assignment.as_ref() {
                    if let Ok(val) = x.input.parse::<u32>() {
                        sql::set_assigned(&self.conn, x.id, val);
                    }
                }
                tree.edit_assignment = None;
                self.refresh();
    };
    Command::none()
        */

    fn view(&self) -> Element<Message> {
        if let Some(page) = self.page.last() {
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
