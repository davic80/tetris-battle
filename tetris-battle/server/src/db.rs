use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS games (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                room_code       TEXT NOT NULL,
                player1_name    TEXT NOT NULL,
                player2_name    TEXT NOT NULL,
                winner_name     TEXT,
                started_at      TEXT NOT NULL,
                ended_at        TEXT,
                duration_secs   INTEGER
            );",
        )?;
        Ok(())
    }

    pub fn save_game(&self, game: &GameRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO games (room_code, player1_name, player2_name, winner_name, started_at, ended_at, duration_secs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                game.room_code,
                game.player1_name,
                game.player2_name,
                game.winner_name,
                game.started_at,
                game.ended_at,
                game.duration_secs
            ],
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GameRecord {
    pub room_code: String,
    pub player1_name: String,
    pub player2_name: String,
    pub winner_name: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_secs: Option<i64>,
}

impl GameRecord {
    pub fn new(room_code: String, p1: String, p2: String) -> Self {
        GameRecord {
            room_code,
            player1_name: p1,
            player2_name: p2,
            winner_name: None,
            started_at: Utc::now().to_rfc3339(),
            ended_at: None,
            duration_secs: None,
        }
    }
}
