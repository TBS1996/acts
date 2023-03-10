use crate::sql;
use crate::ActID;
use crate::Conn;

#[derive(Debug)]
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

    pub fn total_weighted_time_all_activities(conn: &Conn) -> std::time::Duration {
        let sessions =
            sql::query_map(conn, Self::SELECT_QUERY, |row| Session::try_from(row)).unwrap();
        Self::total_weighted_time_from_sessions(&sessions)
    }

    fn total_weighted_time_from_sessions(sessions: &Vec<Session>) -> std::time::Duration {
        let unix_now = crate::utils::current_unix();

        let mut total_time = std::time::Duration::default();

        for session in sessions {
            let diff = std::time::Duration::from_secs(unix_now.as_secs() - session.timestamp);
            let factor = Self::get_decay_factor_from_duration(diff);
            let time = session.duration.mul_f32(factor);
            total_time += time;
        }
        total_time
    }

    pub fn total_weighted_time_spent_from_activity(conn: &Conn, id: ActID) -> std::time::Duration {
        let sessions = Self::get_history(conn, id);
        Self::total_weighted_time_from_sessions(&sessions)
    }

    fn get_decay_factor_from_duration(duration: std::time::Duration) -> f32 {
        let days = duration.as_secs_f32() / 86400.;

        std::f32::consts::E.powf((0.99 as f32).ln() * days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_factor() {
        assert_eq!(
            Session::get_decay_factor_from_duration(std::time::Duration::from_secs(86400)),
            0.99
        );
    }
}
