use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use tracing::{debug, error};

use crate::db::user::UserDbError;
use crate::model::user::{NewUser, User};

use super::ApiContext;

pub struct DbUser{
    pub name: String,
    pub groups: Option<Vec<String>>,
}

impl From<User> for DbUser{
    fn from(value: User) -> Self {
        Self { name: value.name, groups: value.groups }
    }
}

pub async fn users_get(State(state): State<ApiContext>) -> Result<Json<Vec<User>>, StatusCode> {
    let result = state.user_db.read_users().await;
    if result.is_err() {
        result.as_ref().inspect_err(|e| debug!("Error: {}", e));
    }
    match result{
        Ok(users) => Ok(Json(users)),
        Err(UserDbError::NoneFound) => Ok(Json(vec![])),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn users_post(
    State(state): State<ApiContext>,
    Json(user): Json<NewUser>,
) -> Result<Json<User>, StatusCode> {
    let result = state.user_db.create_user(user.email.clone(), user.password.clone(), user.into()).await;
    if result.is_err(){
        error!("error");
        result.as_ref().inspect_err(|e| debug!("Error: {}", e));
    }
    let Ok(user) = result else {
        return Err(StatusCode::BAD_REQUEST);
    };
    Ok(Json(user))
}

pub async fn user_get(
    State(state): State<ApiContext>,
    Path(id): Path<i64>,
) -> Result<Json<User>, StatusCode> {
    error!(id);
    let result = state.user_db.read_user(id).await;
    if result.is_err() {
        error!("error");
        result.as_ref().inspect_err(|e| debug!("Error: {}", e));
        return Err(StatusCode::NOT_FOUND);
    }
    debug!("I'm here");
    let user = result.unwrap();
    Ok(Json(user))
}

pub async fn user_put(
    State(state): State<ApiContext>,
    Path(id): Path<i64>,
    Json(user): Json<User>,
) -> Result<Json<User>, StatusCode> {
    let Ok(user) = state.user_db.update_user(id, user).await else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(user))
}

pub async fn user_patch(
    State(state): State<ApiContext>,
    Path(id): Path<i64>,
    Json(user): Json<User>,
) -> Result<Json<User>, StatusCode> {
    let Ok(user) = state.user_db.update_user(id, user).await else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(user))
}

pub async fn user_delete(
    State(state): State<ApiContext>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let Ok(_) = state.user_db.delete_user(id).await else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(StatusCode::NO_CONTENT)
}
