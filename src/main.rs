use crate::pages::treeview::TreeView;
use iced::widget::{button, column, pick_list, row, text_input};

use iced::widget::Column;
use iced::{Alignment, Element, Renderer, Sandbox, Settings};

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

type Conn = rusqlite::Connection;
type ActID = usize;

#[derive(Debug)]
pub struct App {
    conn: Conn,
    textboxval: String,
    activities: Vec<Activity>,
    page: Page,
}

#[derive(Default, Debug)]
pub enum Page {
    #[default]
    Main,
    Edit(EditPage),
    TreeView(TreeView),
}

impl Page {
    pub fn is_main(&self) -> bool {
        matches!(self, Self::Main)
    }

    pub fn is_edit(&self) -> bool {
        matches!(self, Self::Edit(_))
    }
}

impl App {
    fn view_activities(&self) -> Vec<Element<'static, Message>> {
        let acts = Self::view_by_priority(&self);

        let mut wtf = vec![];

        for act in acts {
            let button: iced::widget::button::Button<Message> =
                iced::widget::button(iced::widget::text::Text::new(act.display_flat(&self.conn)))
                    .on_press(Message::MainEditActivity(act.id));
            let row = iced::Element::new(iced::widget::row![button]);
            wtf.push(row);
        }
        wtf
    }

    fn view_by_priority(&self) -> Vec<Activity> {
        let mut activities = Activity::fetch_all_activities(&self.conn);
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
        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("Add activity", &self.textboxval, Message::MainInputChanged)
                .on_submit(Message::MainAddActivity)
                .padding(20)
                .size(30);
        let refresh_button = button("Refresh").on_press(Message::MainRefresh);
        let treeview_button = button("view tree").on_press(Message::GoToTree);

        column![
            text_input,
            row![refresh_button, treeview_button],
            Column::with_children(self.view_activities())
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
    MainAssigned(String),
    MainAddActivity,
    MainEditActivity(ActID),
    MainNewSession(ActID),
    MainRefresh,
    MainGoUp(ActID),
    MainGoDown(ActID),
    MainGoLeft(ActID),
    MainGoRight(ActID),

    EditDeleteActivity(ActID),
    EditGotoMain,
    EditInputChanged(String),
    EditAssignInput(String),
    EditSessionInput(String),
    EditAddSession,
    EditAddAssign,

    GoToTree,
    PickAct(ActID),
    GoBack,
}

impl Sandbox for App {
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
                Message::MainInputChanged(x) => self.textboxval = x,

                Message::MainAddActivity => {
                    let x: String = std::mem::take(&mut self.textboxval);
                    let activity = Activity::new(&self.conn, x);
                    sql::new_activity(&self.conn, &activity).unwrap();
                    self.activities.push(activity);
                    self.refresh();
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
                Message::MainRefresh => self.refresh(),
                Message::GoToTree => self.page = Page::TreeView(TreeView::new(&self.conn)),

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
                    if let Ok(num) = editor.assigned.parse::<f32>() {
                        let num = num / 100.;
                        assert!(num < 1. && num > 0.);
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
                _ => {
                    panic!("you forgot to add {:?} to this match arm", message)
                } /*
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
                      */
            },
            _ => {}
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.page {
            Page::Main => self.main_view(),
            Page::Edit(page) => page.view(),
            Page::TreeView(page) => page.view_activities(&self.conn),
        }
    }
}
