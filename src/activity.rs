use crate::sql;
use crate::ActID;
use crate::Conn;

#[derive(Clone, Debug)]
pub struct Activity {
    pub id: ActID,
    pub text: String,
    pub priority: f32,
    pub assigned: f32,
    pub parent: Option<ActID>,
    pub children: Vec<Activity>,
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

    pub fn normalize_positions(conn: &Conn, parent: Option<ActID>) {
        let children = Self::fetch_children(conn, parent);

        for (idx, child) in children.iter().enumerate() {
            let statement = format!(
                "UPDATE activities SET position = {} WHERE id = {}",
                idx, child.id
            );
            sql::execute(conn, &statement).unwrap();
        }
    }

    pub fn get_parent(conn: &Conn, id: ActID) -> Option<ActID> {
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

    pub fn new(conn: &Conn, text: String) -> Self {
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
    pub fn display(&self) -> String {
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
