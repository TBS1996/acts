use crate::sql;
use crate::ActID;
use crate::Conn;

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
    const SELECT_QUERY: &str = "SELECT id, duration, timestamp FROM history";

    pub fn get_history(conn: &Conn, id: ActID) -> Vec<Session> {
        let statement = format!(
            "{}  WHERE id = {} ORDER BY timestamp",
            Self::SELECT_QUERY,
            id
        );
        sql::query_map(conn, &statement, |row| Session::try_from(row)).unwrap()
    }

    pub fn total_time_all_activities(conn: &Conn) -> std::time::Duration {
        let sessions =
            sql::query_map(conn, Self::SELECT_QUERY, |row| Session::try_from(row)).unwrap();
        let mut total = std::time::Duration::default();

        for session in sessions {
            total += session.duration;
        }

        total
    }

    pub fn total_time_spent_from_activity(conn: &Conn, id: ActID) -> std::time::Duration {
        let sessions = Self::get_history(conn, id);
        let mut total = std::time::Duration::default();

        for session in sessions {
            total += session.duration;
        }

        total
    }
}
