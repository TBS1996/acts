use crate::pages::treeview::TreeView;
use iced::widget::{button, column, pick_list, row, text_input};

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

    pub fn refresh(&mut self, conn: &Conn) {
        match self {
            Self::Main => {}
            Self::Edit(editview) => editview.refresh(conn),
            Self::TreeView(tree) => tree.refresh(conn),
        }
    }
}

impl App {
    fn view_activities(&self) -> Vec<Element<'static, Message>> {
        let acts = Self::view_by_priority(&self);

        let mut wtf = vec![];

        for act in acts {
            let button: iced::widget::button::Button<Message> =
                iced::widget::button(iced::widget::text::Text::new(act.display_flat(&self.conn)))
                    .on_press(Message::EditActivity(act.id));
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

    fn refresh(&mut self) {
        self.activities = Activity::fetch_all_activities(&self.conn);
        self.textboxval = String::new();
        Activity::normalize_assignments(&self.conn);
        self.page.refresh(&self.conn);
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    MainInputChanged(String),
    MainAssigned(String),
    MainAddActivity,
    EditActivity(ActID),
    MainNewSession(ActID),
    MainRefresh,

    EditDeleteActivity(ActID),
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
    InputChangeIndexed(usize, String),
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
            page: Page::default(),
        };
        (app, Command::none())
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match &mut self.page {
            Page::Main => match message {
                Message::EditActivity(id) => {
                    self.page = Page::Edit(EditPage::new(&self.conn, id));
                }
                Message::MainInputChanged(x) => {
                    self.textboxval = x;
                }

                Message::MainAddActivity => {
                    let x: String = std::mem::take(&mut self.textboxval);
                    let activity = Activity::new(&self.conn, x);
                    sql::new_activity(&self.conn, &activity).unwrap();
                    self.activities.push(activity);
                    self.refresh();
                }
                Message::MainRefresh => {
                    self.refresh();
                }
                Message::GoToTree => {
                    self.page = Page::TreeView(TreeView::new(&self.conn));
                }

                _ => {
                    panic!("you forgot to add {:?} to this match arm", message)
                }
            },
            Page::Edit(editor) => match message {
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
                _ => {
                    panic!("you forgot to add {:?} to this match arm", message)
                }
            },
            Page::TreeView(tree) => match (&tree.picker, message) {
                (None, Message::GoBack) if tree.edit_assignment.is_some() => {
                    tree.edit_assignment = None;
                    self.refresh();
                }
                (None, Message::GoBack) => {
                    self.page = Page::Main;
                    self.refresh();
                }
                (Some(_), Message::GoBack) => {
                    tree.picker = None;
                    self.refresh();
                }
                (Some(x), Message::PickAct(id)) => {
                    Activity::set_parent(&self.conn, x.0, id);
                    tree.picker = None;
                    self.refresh();
                }
                (None, Message::EditActivity(id)) => {
                    self.page = Page::Edit(EditPage::new(&self.conn, id));
                }
                (None, Message::ChooseParent { child }) => {
                    tree.picker = Some((child, Picker::new(&self.conn)))
                }
                (None, Message::GoAssign(id)) => {
                    let x = ValueGetter::new("assign some stuff".to_string(), id);
                    tree.edit_assignment = Some(x);
                }
                (_, Message::ValueGetInput(val)) => {
                    if val.is_empty() || val.parse::<u32>().is_ok() {
                        if let Some(x) = tree.edit_assignment.as_mut() {
                            x.input = val;
                        }
                    }
                }
                (_, Message::SubmitValue) => {
                    if let Some(x) = tree.edit_assignment.as_ref() {
                        if let Ok(val) = x.input.parse::<u32>() {
                            sql::set_assigned(&self.conn, x.id, val);
                        }
                    }
                    tree.edit_assignment = None;
                    self.refresh();
                }
                (_, _) => {}
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match &self.page {
            Page::Main => self.main_view(),
            Page::Edit(page) => page.view(),
            Page::TreeView(page) => page.view_activities(&self.conn),
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
