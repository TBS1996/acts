use crate::sql;
use crate::ActID;
use crate::Conn;
use uuid::Uuid;

#[derive(Debug)]
pub struct Session {
    _id: ActID,
    duration: std::time::Duration,
    timestamp: u64,
}

impl std::convert::TryFrom<&rusqlite::Row<'_>> for Session {
    type Error = rusqlite::Error;

    fn try_from(value: &rusqlite::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: Uuid::parse_str(&value.get::<usize, String>(0)?).unwrap(),
            duration: std::time::Duration::from_secs_f64(value.get::<usize, f64>(1)? * 60.),
            timestamp: value.get(2)?,
        })
    }
}

impl Session {
    const SELECT_QUERY: &str = "SELECT id, duration, timestamp FROM history";

    pub fn get_history(conn: &Conn, id: ActID) -> Vec<Session> {
        let statement = format!(
            "{}  WHERE id = '{}' ORDER BY timestamp",
            Self::SELECT_QUERY,
            id
        );
        sql::query_map(conn, &statement, |row| Session::try_from(row)).unwrap()
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

    fn average_daily_weighted_time_from_sessions(sessions: &Vec<Session>) -> std::time::Duration {
    let unix_now = crate::utils::current_unix();

    let mut total_time = std::time::Duration::default();
    let mut total_decay_factor = 0f32;

    for session in sessions {
        let diff = std::time::Duration::from_secs(unix_now.as_secs() - session.timestamp);
        let decay_factor = Self::get_decay_factor_from_duration(diff);
        let time = session.duration.mul_f32(decay_factor);

        total_time += time;
        total_decay_factor += decay_factor;
    }

    if total_decay_factor == 0f32 {
       return std::time::Duration::default();
    }

    total_time
}

pub fn average_daily_weighted_time_spent_from_activity(conn: &Conn, id: ActID) -> std::time::Duration {
    let sessions = Self::get_history(conn, id);
    Self::average_daily_weighted_time_from_sessions(&sessions)
}


    pub fn total_weighted_time_spent_from_activity(conn: &Conn, id: ActID) -> std::time::Duration {
        let sessions = Self::get_history(conn, id);
        Self::total_weighted_time_from_sessions(&sessions)
    }

    fn get_decay_factor_from_duration(duration: std::time::Duration) -> f32 {
        let days = duration.as_secs_f32() / 86400.;
        std::f32::consts::E.powf(0.99f32.ln() * days)
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
