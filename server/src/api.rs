use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::Utc;

use crate::room::{AppState, PlayerInfo, Room, RoomState};
use crate::db::GameRecord;

// ─── REST API ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateRoomBody {
    pub player_name: String,
}

#[derive(Serialize)]
pub struct CreateRoomResponse {
    pub code: String,
    pub slot: u8,
}

pub async fn create_room(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRoomBody>,
) -> impl IntoResponse {
    let name = body.player_name.trim().to_string();
    if name.is_empty() || name.len() > 20 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"invalid name"}))).into_response();
    }
    // name is stored by the client and used when opening the WebSocket

    let code = generate_code();
    let room = Room {
        code: code.clone(),
        player1: None,
        player2: None,
        state: RoomState::Waiting,
        created_at: Utc::now().to_rfc3339(),
    };

    {
        let mut rooms = state.rooms.lock().unwrap();
        rooms.insert(code.clone(), room);
    }

    // Pre-create channel
    state.get_or_create_channel(&code);

    (StatusCode::CREATED, Json(CreateRoomResponse { code, slot: 1 })).into_response()
}

#[derive(Serialize)]
pub struct GetRoomResponse {
    pub code: String,
    pub state: RoomState,
    pub player1: Option<String>,
    pub player2: Option<String>,
}

pub async fn get_room(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let rooms = state.rooms.lock().unwrap();
    match rooms.get(&code) {
        Some(room) => {
            let resp = GetRoomResponse {
                code: room.code.clone(),
                state: room.state.clone(),
                player1: room.player1.as_ref().map(|p| p.name.clone()),
                player2: room.player2.as_ref().map(|p| p.name.clone()),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"room not found"}))).into_response(),
    }
}

// ─── WebSocket Handler ─────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct JoinMsg {
    #[serde(rename = "type")]
    msg_type: String,
    player_name: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(code): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, code, state))
}

async fn handle_socket(socket: WebSocket, code: String, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Expect first message: join { type: "join", player_name: "..." }
    let join_msg = match receiver.next().await {
        Some(Ok(Message::Text(txt))) => {
            match serde_json::from_str::<JoinMsg>(&txt) {
                Ok(m) if m.msg_type == "join" => m,
                _ => {
                    let _ = sender.send(Message::Text(
                        r#"{"type":"error","msg":"first message must be join"}"#.into()
                    )).await;
                    return;
                }
            }
        }
        _ => return,
    };

    let player_name = join_msg.player_name.trim().to_string();
    if player_name.is_empty() || player_name.len() > 20 {
        let _ = sender.send(Message::Text(
            r#"{"type":"error","msg":"invalid name"}"#.into()
        )).await;
        return;
    }

    // Assign slot — lock, compute result, drop lock, then await if needed
    enum SlotResult { Ok(u8), NotFound, Full }
    let slot_result = {
        let mut rooms = state.rooms.lock().unwrap();
        match rooms.get_mut(&code) {
            None => SlotResult::NotFound,
            Some(room) => {
                if room.player1.is_none() {
                    room.player1 = Some(PlayerInfo { name: player_name.clone(), slot: 1 });
                    SlotResult::Ok(1u8)
                } else if room.player2.is_none() {
                    room.player2 = Some(PlayerInfo { name: player_name.clone(), slot: 2 });
                    SlotResult::Ok(2u8)
                } else {
                    SlotResult::Full
                }
            }
        }
        // MutexGuard dropped here
    };

    let slot = match slot_result {
        SlotResult::NotFound => {
            let _ = sender.send(Message::Text(
                r#"{"type":"error","msg":"room not found"}"#.into()
            )).await;
            return;
        }
        SlotResult::Full => {
            let _ = sender.send(Message::Text(
                r#"{"type":"error","msg":"room full"}"#.into()
            )).await;
            return;
        }
        SlotResult::Ok(s) => s,
    };

    let tx = state.get_or_create_channel(&code);
    let mut rx = tx.subscribe();

    // Notify this player of their slot
    let welcome = serde_json::json!({
        "type": "joined",
        "slot": slot,
        "name": player_name,
        "code": code
    });
    if sender.send(Message::Text(welcome.to_string())).await.is_err() {
        return;
    }

    // Check if both players are now connected
    {
        let mut rooms = state.rooms.lock().unwrap();
        if let Some(room) = rooms.get_mut(&code) {
            if room.player1.is_some() && room.player2.is_some() && room.state == RoomState::Waiting {
                room.state = RoomState::Playing;
                let p1 = room.player1.as_ref().unwrap().name.clone();
                let p2 = room.player2.as_ref().unwrap().name.clone();
                let start_msg = serde_json::json!({
                    "type": "game_start",
                    "player1": p1,
                    "player2": p2
                });
                let _ = tx.send(start_msg.to_string());
            }
        }
    }

    // Spawn task to forward broadcast → websocket sender
    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if sender.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(_) => {} // lagged, skip
            }
        }
    });

    // Forward client → broadcast (relay)
    let code_clone = code.clone();
    let state_clone = state.clone();
    let tx_clone = tx.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(txt) => {
                    // Parse to tag with sender slot
                    if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&txt) {
                        val["from_slot"] = serde_json::json!(slot);
                        let msg_type = val["type"].as_str().unwrap_or("").to_string();

                        // Handle game_over server-side
                        if msg_type == "game_over" {
                            handle_game_over(&val, &code_clone, &state_clone, &tx_clone).await;
                        }

                        let _ = tx_clone.send(val.to_string());
                    }
                }
                Message::Close(_) => break,
                Message::Ping(data) => {
                    // pong handled by axum automatically
                }
                _ => {}
            }
        }

        // Player disconnected
        let disc_msg = serde_json::json!({
            "type": "opponent_disconnected",
            "slot": slot
        });
        let _ = tx_clone.send(disc_msg.to_string());
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

async fn handle_game_over(
    msg: &serde_json::Value,
    code: &str,
    state: &Arc<AppState>,
    tx: &tokio::sync::broadcast::Sender<String>,
) {
    let loser_slot = msg["from_slot"].as_u64().unwrap_or(0) as u8;
    let winner_slot = if loser_slot == 1 { 2u8 } else { 1u8 };

    let (winner_name, p1_name, p2_name) = {
        let rooms = state.rooms.lock().unwrap();
        if let Some(room) = rooms.get(code) {
            let w = if winner_slot == 1 {
                room.player1.as_ref().map(|p| p.name.clone()).unwrap_or_default()
            } else {
                room.player2.as_ref().map(|p| p.name.clone()).unwrap_or_default()
            };
            let p1 = room.player1.as_ref().map(|p| p.name.clone()).unwrap_or_default();
            let p2 = room.player2.as_ref().map(|p| p.name.clone()).unwrap_or_default();
            (w, p1, p2)
        } else {
            return;
        }
    };

    // Broadcast result
    let result_msg = serde_json::json!({
        "type": "game_result",
        "winner": winner_name,
        "winner_slot": winner_slot
    });
    let _ = tx.send(result_msg.to_string());

    // Save to DB
    let now = Utc::now().to_rfc3339();
    let record = GameRecord {
        room_code: code.to_string(),
        player1_name: p1_name,
        player2_name: p2_name,
        winner_name: Some(winner_name),
        started_at: now.clone(),
        ended_at: Some(now),
        duration_secs: None,
    };
    let _ = state.db.save_game(&record);

    // Mark room finished
    let mut rooms = state.rooms.lock().unwrap();
    if let Some(room) = rooms.get_mut(code) {
        room.state = crate::room::RoomState::Finished;
    }
}

fn generate_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    (0..6).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}
