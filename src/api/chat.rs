use axum::{extract::State, http::StatusCode, Json};

use crate::{
    model::chat::{Message, Room},
    AppState,
};

pub async fn chats_get_rooms(State(state): State<AppState>) -> Result<Json<Vec<Room>>, StatusCode> {
    let Ok(rooms) = state.chat_db.select_rooms().await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    Ok(Json(rooms))
}

pub async fn chats_post_room(
    State(state): State<AppState>,
    Json(room): Json<Room>,
) -> Result<Json<Room>, StatusCode> {
    Ok(Json(Room::default()))
}

pub async fn chats_post_room_message(
    State(state): State<AppState>,
) -> Result<Json<Message>, StatusCode> {
    todo!()
}

pub async fn chats_get_room(State(state): State<AppState>) -> Result<Json<Room>, StatusCode> {
    Ok(Json(Room::default()))
}

pub async fn chats_get_room_messages(
    State(state): State<AppState>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    todo!()
}
