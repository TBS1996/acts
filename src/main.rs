use iced::widget::{button, column, text, text_input};
use rusqlite::Connection;

use iced::widget::{Button, Column, Container, Slider};
use iced::{Alignment, Color, Element, Length, Renderer, Sandbox, Settings};
pub fn main() -> iced::Result {
    Counter::run(Settings::default())
}

type Conn = rusqlite::Connection;
type ActID = usize;
const PATH: &str = "mydb.db";

pub struct Counter {
    conn: Conn,
    textboxval: String,
    activities: Vec<Activity>,
    page: Page,
}

pub struct EditPage {
    activity: Activity,
}

impl EditPage {
    fn new(conn: &Conn, id: ActID) -> Self {
        Self {
            activity: Activity::fetch_activity(conn, id).unwrap(),
        }
    }
    pub fn view(&self) -> Element<'static, Message> {
        let text_input: iced::widget::text_input::TextInput<'_, Message, Renderer> =
            text_input("hey", &self.activity.text, Message::EditInputChanged)
                .padding(20)
                .size(30);
        column![
            text_input,
            button("go back to main").on_press(Message::EditGotoMain),
            button("Delete").on_press(Message::EditDeleteActivity(self.activity.id)),
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }
}

#[derive(Default)]
pub enum Page {
    #[default]
    Main,
    Edit(EditPage),
}

#[derive(Clone, Debug)]
pub struct Activity {
    pub id: ActID,
    pub text: String,
    pub priority: f32,
    pub assigned: f32,
    pub parent: Option<ActID>,
    pub children: Vec<Activity>,
}

pub struct Session {
    id: ActID,
    duration: std::time::Duration,
    timestamp: u64,
}

impl std::convert::TryFrom<&rusqlite::Row<'_>> for Session {
    type Error = rusqlite::Error;

    fn try_from(value: &rusqlite::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.get(0)?,
            duration: std::time::Duration::from_secs_f64(value.get::<usize, f64>(1)? as f64 * 60.),
            timestamp: value.get(2)?,
        })
    }
}

impl Session {
    pub fn get_history(conn: &Conn, id: ActID) -> Vec<Session> {
        let statement = format!(
            "SELECT id, duration, timestamp FROM history WHERE id = {} ORDER BY timestamp",
            id
        );
        sql::query_map(conn, &statement, |row| Session::try_from(row)).unwrap()
    }
}

impl std::convert::TryFrom<&rusqlite::Row<'_>> for Activity {
    type Error = rusqlite::Error;

    fn try_from(value: &rusqlite::Row) -> Result<Self, Self::Error> {
        Ok(Activity {
            id: value.get(0)?,
            text: value.get(1)?,
            parent: value.get(2)?,
            assigned: value.get(3)?,
            priority: 1.,
            children: vec![],
        })
    }
}

impl Activity {
    const SELECT_QUERY: &str = "SELECT id, text, parent, assigned, position FROM activities";

    fn query_id(id: ActID) -> String {
        format!("{} WHERE id = {}", Self::SELECT_QUERY, id)
    }

    fn normalize_positions(conn: &Conn, parent: Option<ActID>) {
        let children = Self::fetch_children(conn, parent);

        for (idx, child) in children.iter().enumerate() {
            let statement = format!(
                "UPDATE activities SET position = {} WHERE id = {}",
                idx, child.id
            );
            sql::execute(conn, &statement).unwrap();
        }
    }

    fn get_parent(conn: &Conn, id: ActID) -> Option<ActID> {
        Activity::fetch_activity(conn, id).unwrap().parent
    }

    pub fn get_position(conn: &Conn, id: ActID) -> usize {
        let statement = format!("SELECT position FROM activities WHERE id = {}", id);
        sql::query_row(conn, &statement, |row| {
            Ok(row.get::<usize, usize>(0).unwrap())
        })
        .unwrap()
    }

    pub fn go_down(conn: &Conn, id: ActID) {
        let activity = Activity::fetch_activity(conn, id).unwrap();
        let position = Self::get_position(conn, activity.id);

        let siblings = Self::fetch_children(conn, activity.parent);

        if position == siblings.len() - 1 {
            return;
        }

        let statement = format!(
            "UPDATE activities SET position = {} WHERE id = {}",
            position + 1,
            siblings[position].id
        );
        sql::execute(conn, &statement).unwrap();

        let statement = format!(
            "UPDATE activities SET position = {} WHERE id = {}",
            position,
            siblings[position + 1].id
        );
        sql::execute(conn, &statement).unwrap();
    }

    pub fn set_parent(conn: &Conn, child: ActID, parent: Option<ActID>) {
        let statement = match parent {
            Some(parent) => format!(
                "UPDATE activities SET parent = {} WHERE id = {}",
                parent, child
            ),
            None => format!("UPDATE activities SET parent = NULL WHERE id = {}", child),
        };

        sql::execute(conn, &statement).unwrap();
        Self::normalize_positions(conn, parent);
    }

    pub fn go_right(conn: &Conn, id: ActID) {
        let parent = Self::get_parent(conn, id);
        let position = Activity::get_position(conn, id);
        let siblings = Self::fetch_children(conn, parent);

        if position == siblings.len() - 1 {
            return;
        }

        let new_parent = siblings[position + 1].id;
        Self::set_parent(conn, id, Some(new_parent));
    }

    pub fn go_left(conn: &Conn, id: ActID) {
        let parent = Activity::get_parent(conn, id);
        if let Some(parent) = parent {
            let grandparent = Activity::get_parent(conn, parent);
            Self::set_parent(conn, id, grandparent);
        }
    }

    pub fn go_up(conn: &Conn, id: ActID) {
        let activity = Activity::fetch_activity(conn, id).unwrap();
        let position = Self::get_position(conn, activity.id);

        if position == 0 {
            return;
        }

        let siblings = Self::fetch_children(conn, activity.parent);

        let statement = format!(
            "UPDATE activities SET position = {} WHERE id = {}",
            position - 1,
            siblings[position].id
        );
        sql::execute(conn, &statement).unwrap();

        let statement = format!(
            "UPDATE activities SET position = {} WHERE id = {}",
            position,
            siblings[position - 1].id
        );
        sql::execute(conn, &statement).unwrap();
    }

    /// Queries children, but not recursively.
    pub fn fetch_children(conn: &Conn, parent: Option<ActID>) -> Vec<Activity> {
        sql::query_map(conn, &Self::query_children(parent), |row| {
            Activity::try_from(row)
        })
        .unwrap()
    }

    fn query_children(parent: Option<ActID>) -> String {
        match parent {
            Some(id) => format!(
                "{} WHERE parent = {} ORDER BY position",
                Self::SELECT_QUERY,
                id
            ),
            None => format!(
                "{} WHERE parent IS NULL ORDER BY position",
                Self::SELECT_QUERY
            ),
        }
    }

    fn update_text(conn: &Conn, id: ActID, text: &String) -> Result<(), rusqlite::Error> {
        let statement = format!(
            "UPDATE activities SET text = \"{}\" WHERE id = {}",
            text, id
        );

        sql::execute(conn, &statement)
    }

    pub fn modify_text(&mut self, text: String, conn: &Conn) {
        Self::update_text(conn, self.id, &text).unwrap();
        self.text = text;
    }

    fn new(conn: &Conn, text: String) -> Self {
        let id = sql::get_card_qty(&conn);
        Self {
            id,
            text,
            priority: 1.,
            assigned: 1.,
            parent: None,
            children: vec![],
        }
    }
    fn display(&self) -> String {
        format!("{} -> {}", self.text, self.priority)
    }

    fn push_child(child: Activity, activities: &mut Vec<Activity>, parent: Option<ActID>) {
        match parent {
            Some(parent) => {
                for activity in activities {
                    if activity.id == parent {
                        activity.children.push(child);
                        return;
                    }
                }
            }
            None => activities.push(child),
        };
    }

    pub fn fetch_activity(conn: &Conn, id: ActID) -> Result<Activity, rusqlite::Error> {
        sql::query_row(conn, &Activity::query_id(id), |row| Self::try_from(row))
    }

    pub fn fetch_activity_by_condition(conn: &Conn, condition: &str) -> Activity {
        let statement = format!("{} WHERE {}", Self::SELECT_QUERY, condition);
        sql::query_row(conn, &statement, |row| Self::try_from(row)).unwrap()
    }

    pub fn fetch_all_activities(conn: &Conn) -> Vec<Activity> {
        let mut activities = vec![];
        Self::fetch_all_activities_helper(conn, &mut activities, None);
        dbg!(&activities);
        activities
    }

    fn fetch_all_activities_helper(
        conn: &Conn,
        activities: &mut Vec<Activity>,
        parent: Option<ActID>,
    ) {
        let statement = Self::query_children(parent);
        let the_activities = sql::query_map(conn, &statement, |row: &rusqlite::Row| {
            Activity::try_from(row)
        })
        .unwrap();

        for activity in the_activities {
            let parent = activity.parent;
            let id = activity.id;
            Self::push_child(activity, activities, parent);
            Self::fetch_all_activities_helper(conn, activities, Some(id));
        }
    }
}

impl Counter {
    fn view_helper(activity: Activity, elms: &mut Vec<Element<'static, Message>>, depth: usize) {
        let padding = std::iter::repeat(' ').take(depth * 6).collect::<String>();

        let padding = iced::Element::new(iced::widget::text::Text::new(padding));

        let elm = iced::Element::new(iced::widget::text::Text::new(activity.display()));

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
                .on_press(Message::MainEditActivity(activity.id));
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
            Self::view_helper(activity, elms, depth + 1);
        }
    }

    fn view_activities(&self) -> Vec<Element<'static, Message>> {
        let mut some_vec = Vec::new();
        for act in &self.activities {
            Self::view_helper(act.clone(), &mut some_vec, 0);
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

    fn refresh(&mut self) {
        self.activities = Activity::fetch_all_activities(&self.conn);
        self.textboxval = String::new();
    }
}

#[derive(Default)]
struct MainPage;

#[derive(Debug, Clone)]
pub enum Message {
    MainInputChanged(String),
    MainAddActivity,
    MainEditActivity(ActID),
    MainGoUp(ActID),
    MainGoDown(ActID),
    MainGoLeft(ActID),
    MainGoRight(ActID),

    EditDeleteActivity(ActID),
    EditGotoMain,
    EditInputChanged(String),

    AddSession {
        id: ActID,
        duration: std::time::Duration,
    },
}

impl Sandbox for Counter {
    type Message = Message;

    fn new() -> Self {
        let conn = sql::init();
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
                    let parent = Activity::get_parent(&self.conn, id);
                    sql::delete_activity(&self.conn, id);
                    self.page = Page::Main;
                    Activity::normalize_positions(&self.conn, parent);
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
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.page {
            Page::Main => self.main_view(),
            Page::Edit(page) => page.view(),
        }
    }
}

mod sql {
    use super::*;

    pub fn get_card_qty(conn: &Conn) -> usize {
        let statement = "SELECT COUNT(*) FROM activities";

        if conn
            .query_row(statement, [], |row| row.get::<usize, usize>(0))
            .unwrap()
            == 0
        {
            return 0;
        }
        let statement = "SELECT MAX(id) FROM activities";
        conn.query_row(statement, [], |row| row.get::<usize, usize>(0))
            .unwrap()
            + 1
    }

    pub fn get_kid_qty(conn: &Conn, parent: &Option<ActID>) -> usize {
        let statement = match parent {
            Some(id) => {
                format!("SELECT COUNT(*) FROM activities WHERE parent = {}", id)
            }
            None => "SELECT COUNT(*) FROM activities WHERE parent IS NULL".to_string(),
        };

        conn.query_row(&statement, [], |row| row.get::<usize, usize>(0))
            .unwrap()
    }

    pub fn execute(conn: &Conn, statement: &str) -> Result<(), rusqlite::Error> {
        conn.execute(statement, [])?;
        Ok(())
    }

    pub fn delete_activity(conn: &Conn, id: ActID) {
        let statement = format!("DELETE FROM activities WHERE id = {}", id);
        execute(conn, &statement).unwrap();
    }

    pub fn init() -> Conn {
        let conn = Connection::open(PATH).unwrap();

        let statement = "CREATE TABLE IF NOT EXISTS activities (
            id INTEGER PRIMARY KEY,
            text TEXT NOT NULL,
            parent INTEGER,
            assigned FLOAT NOT NULL,
            position INTEGER,
            FOREIGN KEY (parent) REFERENCES activities (id)
            )
            ";
        execute(&conn, statement).unwrap();

        let statement = "CREATE TABLE IF NOT EXISTS history (
            id INTEGER,
            minutes FLOAT,
            timestamp INTEGER,
            FOREIGN KEY (id) REFERENCES activities (id)
            )
            ";
        execute(&conn, statement).unwrap();

        conn
    }

    pub fn new_activity(conn: &Conn, activity: &Activity) -> Result<(), rusqlite::Error> {
        let sibqty = get_kid_qty(conn, &activity.parent);
        conn.execute(
            "INSERT INTO activities (id, text, parent, assigned, position) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&activity.id, &activity.text, &activity.parent, &activity.assigned, sibqty),
        )?;

        Ok(())
    }

    /// Generic function that queries the database
    /// and transforms it to a vector of some kind of type.
    pub fn query_map<T, F>(
        conn: &Conn,
        statement: &str,
        mut transformer: F,
    ) -> Result<Vec<T>, rusqlite::Error>
    where
        F: FnMut(&rusqlite::Row) -> Result<T, rusqlite::Error>,
    {
        let mut vec = Vec::new();
        conn.prepare(statement)?
            .query_map([], |row| {
                vec.push(transformer(row)?);
                Ok(())
            })?
            .for_each(|_| {});
        Ok(vec)
    }

    /// Generic function that queries any row from the database
    /// and transforms it to some kind of type.
    pub fn query_row<T, F>(
        conn: &Conn,
        statement: &str,
        mut transformer: F,
    ) -> Result<T, rusqlite::Error>
    where
        F: FnMut(&rusqlite::Row) -> Result<T, rusqlite::Error>,
    {
        conn.query_row(statement, [], |row| transformer(row))
    }
}
