use crate::activity::Activity;
use crate::ActID;
use crate::Conn;

const PATH: &str = "mydb.db";

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
    let conn = Conn::open(PATH).unwrap();

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
            duration FLOAT,
            timestamp INTEGER,
            FOREIGN KEY (id) REFERENCES activities (id)
            )
            ";
    execute(&conn, statement).unwrap();

    conn
}

pub fn new_activity(conn: &Conn, activity: &Activity) -> Result<(), rusqlite::Error> {
    let sibqty = get_kid_qty(conn, &activity.parent);
    let assigned = if sibqty > 0 { 1. / (sibqty as f32) } else { 1. };
    conn.execute(
        "INSERT INTO activities (id, text, parent, assigned, position) VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &activity.id,
            &activity.text,
            &activity.parent,
            assigned,
            sibqty,
        ),
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
