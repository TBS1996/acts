use crate::activity::Activity;
use crate::ActID;
use crate::Conn;


pub fn get_kid_qty(conn: &Conn, parent: &Option<ActID>) -> usize {
    let statement = match parent {
        Some(id) => {
            format!("SELECT COUNT(*) FROM activities WHERE parent = '{}'", id)
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
    let statement = format!("DELETE FROM activities WHERE id = '{}'", id);
    execute(conn, &statement).unwrap();
}

pub fn set_assigned(conn: &Conn, id: ActID, assigned: u32) {
    let statement = format!(
        "UPDATE activities SET assigned = '{}' WHERE id = '{}'",
        assigned, id
    );
    execute(conn, &statement).unwrap();
}

fn get_db_path() -> std::path::PathBuf{

    let mut file_path = std::path::PathBuf::new();

    if let Some(home_dir) = dirs::home_dir() {
        file_path.push(home_dir);
        file_path.push(".local/share/acts/");
        std::fs::create_dir_all(&file_path).expect("Failed to create acts directory");
        file_path.push("mydb.db");
        return file_path
    } else {panic!()}
}


pub fn init() -> Conn {
    let path = get_db_path();
    let conn = std::rc::Rc::new(rusqlite::Connection::open(path).unwrap());

    let statement = "CREATE TABLE IF NOT EXISTS activities (
            id TEXT NOT NULL,
            text TEXT NOT NULL,
            parent TEXT,
            assigned INTEGER NOT NULL,
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
    let _sibqty = get_kid_qty(conn, &activity.parent);
    //let assigned = if sibqty > 0 { 1. / (sibqty as f32) } else { 1. };
    let assigned = 50;
    conn.execute(
        "INSERT INTO activities (id, text, parent, assigned) VALUES (?1, ?2, ?3, ?4)",
        (
            activity.id.to_string(),
            &activity.text,
            activity.parent.map(|p| p.to_string()),
            assigned,
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
