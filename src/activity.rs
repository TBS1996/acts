use crate::sql;
use crate::ActID;
use crate::Conn;

#[derive(Clone, Debug)]
pub struct Activity {
    pub id: ActID,
    pub text: String,
    pub priority: f32,
    pub assigned: u32,
    pub parent: Option<ActID>,
    pub children: Vec<Activity>,
}

impl std::convert::TryFrom<&rusqlite::Row<'_>> for Activity {
    type Error = rusqlite::Error;

    fn try_from(value: &rusqlite::Row) -> Result<Self, Self::Error> {
        Ok(Activity {
            id: value.get(0).unwrap(),
            text: value.get(1).unwrap(),
            parent: value.get(2).unwrap(),
            assigned: value.get::<usize, u32>(3).unwrap(),
            priority: 1.,
            children: vec![],
        })
    }
}

impl Activity {
    const SELECT_QUERY: &str = "SELECT id, text, parent, assigned FROM activities";

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

    pub fn get_parent_index(conn: &Conn, id: ActID) -> Option<ActID> {
        Activity::fetch_activity(conn, id).unwrap().parent
    }

    pub fn get_parent(conn: &Conn, id: ActID) -> Option<Activity> {
        let index = Activity::get_parent_index(conn, id)?;
        Activity::fetch_activity(conn, index).ok()
    }

    pub fn set_parent(conn: &Conn, child: ActID, parent: Option<ActID>) {
        let statement = match parent {
            Some(parent) => {
                if parent == child {
                    return;
                }

                if Activity::fetch_activity(conn, parent).unwrap().parent == Some(child) {
                    return;
                }

                format!(
                    "UPDATE activities SET parent = {} WHERE id = {}",
                    parent, child
                )
            }
            None => format!("UPDATE activities SET parent = NULL WHERE id = {}", child),
        };

        sql::execute(conn, &statement).unwrap();
    }

    pub fn get_true_assigned(conn: &Conn, mut id: ActID) -> f32 {
        let assigned = Activity::fetch_activity(conn, id).unwrap().assigned as f32;
        let mut multiply = 1.;

        while let Some(parent) = Activity::get_parent(conn, id) {
            multiply *= (parent.assigned as f32) / 100.;
            id = parent.id;
        }

        assigned * multiply
    }

    /// Queries children, but not recursively.
    pub fn fetch_children(conn: &Conn, parent: Option<ActID>) -> Vec<Activity> {
        dbg!(&parent);
        dbg!(sql::query_map(conn, &Self::query_children(parent), |row| {
            Activity::try_from(row)
        })
        .unwrap())
    }

    fn query_children(parent: Option<ActID>) -> String {
        match parent {
            Some(id) => format!("{} WHERE parent = {}", Self::SELECT_QUERY, id),
            None => format!("{} WHERE parent IS NULL", Self::SELECT_QUERY),
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
        let id = sql::get_card_qty(conn);
        Self {
            id,
            text,
            priority: 1.,
            assigned: 50,
            parent: None,
            children: vec![],
        }
    }

    pub fn display_flat(&self, conn: &Conn) -> String {
        format!(
            "{}:  {:.1}",
            self.text,
            Self::calculate_priority(conn, self.id).powf(0.5)
        )
    }

    pub fn calculate_priority(conn: &Conn, id: ActID) -> f32 {
        let total = crate::history::Session::total_weighted_time_all_activities(conn);
        let time_spent = crate::history::Session::total_weighted_time_spent_from_activity(conn, id);

        let ratio = (time_spent.as_secs_f32() / 60. + 1.) / (total.as_secs_f32() / 60. + 1.);

        Activity::get_true_assigned(conn, id) / ratio
    }

    pub fn fetch_activity(conn: &Conn, id: ActID) -> Result<Activity, rusqlite::Error> {
        sql::query_row(conn, &Activity::query_id(id), |row| Self::try_from(row))
    }

    pub fn assign_priorities(conn: &Conn, activities: &mut Vec<Activity>) {
        let mut f = |conn: &Conn, activity: &mut Activity| {
            activity.priority = Self::calculate_priority(conn, activity.id);
        };

        Self::activity_walker_dfs(conn, activities, &mut f);
    }

    pub fn fetch_all_activities_flat(conn: &Conn) -> Vec<Activity> {
        sql::query_map(conn, Activity::SELECT_QUERY, |row: &rusqlite::Row| {
            Activity::try_from(row)
        })
        .unwrap()
    }

    pub fn delete_activity(conn: &Conn, id: ActID) {
        // Before deleting activity, make sure all the children get the parent of the to-be-deleted
        // activity.
        let activity = Self::fetch_activity(conn, id).unwrap();
        let parent = activity.parent;

        let children = Self::fetch_children(conn, Some(id));

        for child in children {
            Self::set_parent(conn, child.id, parent);
        }

        sql::delete_activity(conn, id);
    }

    pub fn normalize_assignments(conn: &Conn) {
        fn recursive(conn: &Conn, parent: Option<ActID>) {
            let kids = Activity::fetch_children(conn, parent);
            if !kids.is_empty() {
                let nums = kids.iter().map(|kid| kid.assigned as i32).collect();
                let normalized = crate::utils::normalize_vec(nums, 100);

                for (idx, kid) in kids.iter().enumerate() {
                    sql::set_assigned(conn, kid.id, normalized[idx] as u32);
                    recursive(conn, Some(kid.id));
                }
            }
        }
        recursive(conn, None);
    }
}
