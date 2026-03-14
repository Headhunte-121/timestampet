#[cfg(test)]
mod tests_feature_5_12 {
    use super::*;
    use rusqlite::{Connection, params};

    #[test]
    fn test_fresh_library_zero_default() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE Episodes (id INTEGER PRIMARY KEY, last_position INTEGER NOT NULL DEFAULT 0)", ()).unwrap();
        conn.execute("INSERT INTO Episodes DEFAULT VALUES", ()).unwrap();
        let pos: i32 = conn.query_row("SELECT last_position FROM Episodes LIMIT 1", [], |r| r.get(0)).unwrap();
        assert_eq!(pos, 0);
    }
}
