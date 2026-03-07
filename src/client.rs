use crate::bot::Bot;
use crate::enums::{Action, CellType};
use crate::helpers::GameHelpers;
use crate::models::{Bomb, Field, GameState, Player, Position};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use tungstenite::client::IntoClientRequest;
use tungstenite::{Message, connect};

const MSG_WELCOME: &str = "welcome";
const MSG_BACK_TO_LOBBY: &str = "back_to_lobby";
const MSG_UPDATE_LOBBY: &str = "update_lobby";
const MSG_PLAYER_STATUS_UPDATE: &str = "player_status_update";
const MSG_SERVER_ERROR: &str = "error";
const MSG_CLASSIC_INPUT: &str = "classic_input";
const MSG_CLASSIC_STATE: &str = "classic_state";
const MSG_GAME_START: &str = "game_start";

#[derive(Deserialize)]
struct EnvelopeIn {
    #[serde(rename = "type")]
    msg_type: String,
    payload: Value,
}

#[derive(Serialize)]
struct EnvelopeOut<T> {
    #[serde(rename = "type")]
    msg_type: String,
    payload: T,
}

#[derive(Deserialize)]
struct WelcomePayload {
    #[serde(rename = "clientId")]
    client_id: String,
}

#[derive(Deserialize)]
struct ErrorPayload {
    message: String,
}

#[derive(Serialize)]
struct PlayerStatusUpdatePayload {
    #[serde(rename = "isReady")]
    is_ready: bool,
    #[serde(rename = "authToken", skip_serializing_if = "Option::is_none")]
    auth_token: Option<String>,
}

#[derive(Serialize)]
struct ClassicInputPayload {
    #[serde(rename = "move")]
    move_action: Action,
}

#[derive(Deserialize)]
struct ClassicStatePayload {
    players: Vec<Player>,
    field: FieldWire,
    bombs: Vec<Bomb>,
    explosions: Vec<Position>,
}

#[derive(Deserialize)]
struct FieldWire {
    width: i32,
    height: i32,
    field: Vec<CellType>,
}

pub fn run<B: Bot>(mut user_bot: B) {
    let ws_url =
        env::var("BOMBAHEAD_WS_URL").unwrap_or_else(|_| "ws://localhost:8038/ws".to_string());

    let token = env::var("BOMBAHEAD_TOKEN")
        .or_else(|_| env::var("BOMBERMAN_CLIENT_AUTH_TOKEN"))
        .unwrap_or_else(|_| "dev-token-local".to_string());

    println!("Connecting to {}...", ws_url);

    let mut request = ws_url
        .as_str()
        .into_client_request()
        .expect("Invalid Request");
    request.headers_mut().insert(
        "Authorization",
        format!("Bearer {}", token).parse().unwrap(),
    );

    let (mut socket, _) = connect(request).expect("Failed to connect");

    let init_msg = EnvelopeOut {
        msg_type: MSG_PLAYER_STATUS_UPDATE.to_string(),
        payload: PlayerStatusUpdatePayload {
            is_ready: true,
            auth_token: Some(token),
        },
    };
    socket
        .send(Message::Text(
            serde_json::to_string(&init_msg).unwrap().into(),
        ))
        .expect("Failed to send initial ready state");

    let mut my_id = String::new();

    loop {
        let msg = match socket.read() {
            Ok(m) => m,
            Err(e) => {
                println!("Error reading message: {}", e);
                return;
            }
        };

        let text = match msg {
            Message::Text(t) => t,
            _ => continue,
        };

        let env_in: EnvelopeIn = match serde_json::from_str(text.as_str()) {
            Ok(e) => e,
            Err(e) => {
                println!("Invalid message envelope: {}", e);
                continue;
            }
        };

        match env_in.msg_type.as_str() {
            MSG_WELCOME => {
                if let Ok(payload) = serde_json::from_value::<WelcomePayload>(env_in.payload) {
                    my_id = payload.client_id;
                    println!("Connected as {}", my_id);
                }
            }
            MSG_UPDATE_LOBBY => continue,
            MSG_SERVER_ERROR => {
                let payload_clone = env_in.payload.clone();
                if let Ok(payload) = serde_json::from_value::<ErrorPayload>(env_in.payload) {
                    println!("Server error: {}", payload.message);
                } else {
                    println!("Server error (unparsed payload): {}", payload_clone);
                }
            }
            MSG_GAME_START => {
                println!("Game started");
            }
            MSG_BACK_TO_LOBBY => {
                let msg = EnvelopeOut {
                    msg_type: MSG_PLAYER_STATUS_UPDATE.to_string(),
                    payload: PlayerStatusUpdatePayload {
                        is_ready: true,
                        auth_token: None,
                    },
                };
                if socket
                    .send(Message::Text(serde_json::to_string(&msg).unwrap().into()))
                    .is_err()
                {
                    println!("Failed to re-ready in lobby");
                    return;
                }
            }
            MSG_CLASSIC_STATE => {
                let mut payload: ClassicStatePayload = match serde_json::from_value(env_in.payload)
                {
                    Ok(p) => p,
                    Err(e) => {
                        println!("Failed to parse classic state: {}", e);
                        continue;
                    }
                };

                let expected_cells = (payload.field.width * payload.field.height) as usize;
                payload.field.field.resize(expected_cells, CellType::Air);

                let field = Field {
                    width: payload.field.width,
                    height: payload.field.height,
                    cells: payload.field.field,
                };

                let mut me = None;
                let mut opponents = Vec::new();

                for p in &payload.players {
                    if p.id == my_id {
                        me = Some(p.clone());
                    } else {
                        opponents.push(p.clone());
                    }
                }

                if me.is_none() && !payload.players.is_empty() {
                    let p = payload.players[0].clone();
                    me = Some(p);
                    opponents = payload.players.iter().skip(1).cloned().collect();
                }

                let state = GameState {
                    current_tick: 0,
                    me,
                    opponents,
                    players: payload.players,
                    field,
                    bombs: payload.bombs,
                    explosions: payload.explosions,
                };

                let helpers = GameHelpers::new(&state);
                let action = user_bot.get_next_move(&state, &helpers);

                let out_msg = EnvelopeOut {
                    msg_type: MSG_CLASSIC_INPUT.to_string(),
                    payload: ClassicInputPayload {
                        move_action: action,
                    },
                };

                if socket
                    .send(Message::Text(
                        serde_json::to_string(&out_msg).unwrap().into(),
                    ))
                    .is_err()
                {
                    println!("Failed to send action");
                    return;
                }
            }
            _ => {
                println!("Ignoring message type \"{}\"", env_in.msg_type);
            }
        }
    }
}
