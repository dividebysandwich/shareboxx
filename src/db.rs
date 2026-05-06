use rusqlite::{params, Connection};

const DB_FILE: &str = "uploads.db";

pub fn open() -> rusqlite::Result<Connection> {
    let conn = Connection::open(DB_FILE)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS uploads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            rel_path TEXT NOT NULL UNIQUE,
            uploaded_at INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

pub fn record_upload(conn: &Connection, rel_path: &str, ts: u64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO uploads (rel_path, uploaded_at) VALUES (?1, ?2)
         ON CONFLICT(rel_path) DO UPDATE SET uploaded_at = excluded.uploaded_at",
        params![rel_path, ts as i64],
    )?;
    Ok(())
}

pub fn list_tracked(conn: &Connection) -> rusqlite::Result<Vec<(i64, String, u64)>> {
    let mut stmt = conn.prepare("SELECT id, rel_path, uploaded_at FROM uploads ORDER BY uploaded_at ASC")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)? as u64,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn delete_by_id(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM uploads WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn delete_by_path(conn: &Connection, rel_path: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM uploads WHERE rel_path = ?1", params![rel_path])?;
    Ok(())
}

pub fn update_path(conn: &Connection, old: &str, new: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE uploads SET rel_path = ?1 WHERE rel_path = ?2",
        params![new, old],
    )?;
    Ok(())
}
