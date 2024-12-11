use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use json_patch::Patch;
use serde_json::{from_str, json, Value};
use tracing::{debug, error};

use crate::db::user::UserDbError;
use crate::model::subject::{AuthContext, Subject};
use crate::model::user::{NewUser, User};

use super::abac::Decision;
use crate::AppState;

#[tracing::instrument(level = "info")]
pub async fn users_get(State(state): State<AppState>) -> Result<Json<Vec<User>>, StatusCode> {
    let result = state.user_db.read_users().await;
    match result.inspect_err(|e| debug!("Error: {}", e)) {
        Ok(users) => Ok(Json(users)),
        Err(UserDbError::NoneFound) => Ok(Json(vec![])),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[tracing::instrument(level = "info")]
pub async fn users_post(
    State(state): State<AppState>,
    Json(user): Json<NewUser>,
) -> Result<Json<User>, StatusCode> {
    let result = state
        .user_db
        .create_user(user.email.clone(), user.password.clone(), user.into())
        .await;
    match result.inspect_err(|e| debug!("Error: {}", e)) {
        Ok(user) => Ok(Json(user)),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[tracing::instrument(level = "info")]
pub async fn user_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>, StatusCode> {
    error!(id);
    let result = state.user_db.read_user(id).await;
    match result.inspect_err(|e| debug!("Error: {}", e)) {
        Ok(user) => Ok(Json(user)),
        Err(UserDbError::NoneFound) => Err(StatusCode::NOT_FOUND),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[tracing::instrument(level = "info")]
pub async fn user_put(
    State(state): State<AppState>,
    auth_ctx: AuthContext,
    Path(id): Path<i64>,
    Json(mut user): Json<User>,
) -> Result<Json<User>, StatusCode> {
    auth_ctx
        .enforce_policy(|actx| {
            if actx.subject == Subject::UserId(1) {
                return Decision::Permit;
            }
            Decision::Deny
        })
        .map_err(|_| StatusCode::FORBIDDEN)?;
    user.id = Some(id);
    let result = state.user_db.update_user(id, user).await;
    match result.inspect_err(|e| debug!("Error: {}", e)) {
        Ok(user) => Ok(Json(user)),
        Err(UserDbError::NoneFound) => Err(StatusCode::NOT_FOUND),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[tracing::instrument(level = "info")]
pub async fn user_patch(
    State(state): State<AppState>,
    auth_ctx: AuthContext,
    headers: HeaderMap,
    Path(id): Path<i64>,
    body: String,
) -> Result<Json<User>, StatusCode> {
    auth_ctx
        .enforce_policy(|actx| {
            if actx.subject == Subject::UserId(1) {
                return Decision::Permit;
            }
            Decision::Deny
        })
        .map_err(|_| StatusCode::FORBIDDEN)?;
    let Ok(user) = state.user_db.read_user(id).await else {
        return Err(StatusCode::NOT_FOUND);
    };
    let mut doc: Value = json!(&user);
    match headers.get("content-type") {
        Some(hv) if hv == HeaderValue::from_static("application/json-patch+json") => {
            let patch: Patch = from_str(&body).map_err(|e| {
                error!(error= %e);
                StatusCode::BAD_REQUEST
            })?;
            json_patch::patch(&mut doc, &patch).map_err(|e| {
                error!(error= %e);
                StatusCode::BAD_REQUEST
            })?;
        }
        Some(hv) if hv == HeaderValue::from_static("application/json") => {
            let patch: Value = from_str(&body).map_err(|e| {
                error!(error= %e);
                StatusCode::BAD_REQUEST
            })?;
            json_patch::merge(&mut doc, &patch);
        }
        _ => return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE),
    }
    let mut user: User = doc.try_into().map_err(|e| {
        error!(error= %e);
        StatusCode::BAD_REQUEST
    })?;
    user.id = Some(id);
    let result = state.user_db.update_user(id, user).await;
    match result.inspect_err(|e| debug!("Error: {}", e)) {
        Ok(user) => Ok(Json(user)),
        Err(UserDbError::NoneFound) => Err(StatusCode::NOT_FOUND),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[tracing::instrument(level = "info")]
pub async fn user_delete(
    State(state): State<AppState>,
    auth_ctx: AuthContext,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    auth_ctx
        .enforce_policy(|actx| {
            if actx.subject == Subject::UserId(1) {
                return Decision::Permit;
            }
            Decision::Deny
        })
        .map_err(|_| StatusCode::FORBIDDEN)?;
    let Ok(_) = state.user_db.delete_user(id).await else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(StatusCode::NO_CONTENT)
}
