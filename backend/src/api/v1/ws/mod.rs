use std::sync::Arc;

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use log::info;

use crate::{middleware::auth::AuthenticatedUser, state::AppState};

pub async fn ws_handler(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(app_state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, user_id, app_state))
}

async fn handle_socket(mut socket: WebSocket, user_id: String, _app_state: Arc<AppState>) {
    // now you have:
    // - authenticated user email
    // - entire application state

    info!("Websocket connected: {}", user_id);

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(t) => {
                let reply = format!("{} said: {}", user_id, t);
                let _ = socket.send(Message::Text(reply.into())).await;
            }
            Message::Close(_) => {
                info!("Websocket disconnected: {}", user_id);
                break;
            }
            _ => {}
        }
    }
}
