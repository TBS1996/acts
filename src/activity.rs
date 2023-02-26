use std::vec::IntoIter;

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

    /// Iterates over a vector of activities recursively and applies a closure to each of them.
    pub fn activity_walker_dfs<F>(conn: &Conn, activities: &mut Vec<Activity>, f: &mut F)
    where
        F: FnMut(&Conn, &mut Activity),
    {
        fn recursion<F>(conn: &Conn, activity: &mut Activity, f: &mut F)
        where
            F: FnMut(&Conn, &mut Activity),
        {
            f(conn, activity); // This is where the magic happens.

            for child in activity.children.iter_mut() {
                recursion(conn, child, f);
            }
        }

        for activity in activities {
            recursion(conn, activity, f);
        }
    }

    fn query_id(id: ActID) -> String {
        format!("{} WHERE id = {}", Self::SELECT_QUERY, id)
    }

    pub fn normalize_positions(conn: &Conn) {
        let mut activities = Activity::fetch_all_activities(conn);

        let mut f = |conn: &Conn, activity: &mut Activity| {
            for (idx, child) in activity.children.iter().enumerate() {
                let statement = format!(
                    "UPDATE activities SET position = {} WHERE id = {}",
                    idx, child.id
                );
                sql::execute(conn, &statement).unwrap();
            }
        };

        for (idx, child) in activities.iter().enumerate() {
            let statement = format!(
                "UPDATE activities SET position = {} WHERE id = {}",
                idx, child.id
            );
            sql::execute(conn, &statement).unwrap();
        }

        Self::activity_walker_dfs(conn, &mut activities, &mut f);
    }

    pub fn get_parent_index(conn: &Conn, id: ActID) -> Option<ActID> {
        Activity::fetch_activity(conn, id).unwrap().parent
    }

    pub fn get_parent(conn: &Conn, id: ActID) -> Option<Activity> {
        let index = Activity::get_parent_index(conn, id)?;
        Activity::fetch_activity(conn, index).ok()
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
        Self::normalize_positions(conn);
    }

    pub fn get_true_assigned(conn: &Conn, mut id: ActID) -> f32 {
        let assigned = Activity::fetch_activity(conn, id).unwrap().assigned;
        let mut multiply = 1.;

        while let Some(parent) = Activity::get_parent(conn, id) {
            multiply *= parent.assigned;
            id = parent.id;
        }

        assigned * multiply
    }

    pub fn normalize_assignments(conn: &Conn, parent: Option<ActID>) {
        let siblings = Self::fetch_children(conn, parent);

        let mut total_assignment = 0.;

        for sibling in &siblings {
            total_assignment += sibling.assigned;
        }

        for sibling in siblings {
            let new_assignment = sibling.assigned / total_assignment;
            let statement = format!(
                "UPDATE activities SET assigned = {} WHERE id = {}",
                new_assignment, sibling.id
            );
            conn.execute(&statement, []).unwrap();
        }
    }

    pub fn go_right(conn: &Conn, id: ActID) {
        let parent = Self::get_parent_index(conn, id);
        let position = Activity::get_position(conn, id);
        let siblings = Self::fetch_children(conn, parent);

        if position == siblings.len() - 1 {
            return;
        }

        let new_parent = siblings[position + 1].id;
        Self::set_parent(conn, id, Some(new_parent));
    }

    pub fn go_left(conn: &Conn, id: ActID) {
        let parent = Activity::get_parent_index(conn, id);
        if let Some(parent) = parent {
            let grandparent = Activity::get_parent_index(conn, parent);
            Self::set_parent(conn, id, grandparent);
        }
    }

    pub fn go_up(conn: &Conn, id: ActID) {
        Activity::normalize_positions(conn);
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
    pub fn display(&self, conn: &Conn) -> String {
        let assigned = Activity::get_true_assigned(conn, self.id) * 100.;

        format!(
            "{} -> {}%. pri: {}",
            self.text,
            assigned,
            Self::calculate_priority(conn, self.id)
        )
    }

    pub fn calculate_priority(conn: &Conn, id: ActID) -> f32 {
        let total = crate::history::Session::total_time_all_activities(conn);
        let time_spent = crate::history::Session::total_weighted_time_spent_from_activity(conn, id);

        let ratio = (time_spent.as_secs_f32() / 60. + 1.) / (total.as_secs_f32() / 60. + 1.);

        dbg!(&total, &time_spent, &ratio);

        1. / (ratio * Activity::get_true_assigned(conn, id))
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
