use crate::db::Database;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub code: String,
    pub player1: Option<PlayerInfo>,
    pub player2: Option<PlayerInfo>,
    pub state: RoomState,
    pub created_at: String,
    /// Lines cleared by player 1 this game
    pub p1_lines: u32,
    /// Lines cleared by player 2 this game
    pub p2_lines: u32,
    /// Current shared global level (starts at 1, increases every 10 total lines)
    pub global_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RoomState {
    Waiting,  // waiting for player 2
    Ready,    // both connected, game starting
    Playing,  // game in progress
    Finished, // game over
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub name: String,
    pub slot: u8, // 1 or 2
}

pub struct AppState {
    pub rooms: Mutex<HashMap<String, Room>>,
    pub channels: Mutex<HashMap<String, broadcast::Sender<String>>>,
    pub db: Database,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        AppState {
            rooms: Mutex::new(HashMap::new()),
            channels: Mutex::new(HashMap::new()),
            db,
        }
    }

    pub fn get_or_create_channel(&self, code: &str) -> broadcast::Sender<String> {
        let mut channels = self.channels.lock().unwrap();
        if let Some(tx) = channels.get(code) {
            tx.clone()
        } else {
            let (tx, _) = broadcast::channel(64);
            channels.insert(code.to_string(), tx.clone());
            tx
        }
    }

    pub fn cleanup_room(&self, code: &str) {
        let mut rooms = self.rooms.lock().unwrap();
        rooms.remove(code);
        let mut channels = self.channels.lock().unwrap();
        channels.remove(code);
    }
}
