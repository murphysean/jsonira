use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::{model::subject::AuthContext, AppState};

#[tracing::instrument(level = "info")]
pub async fn user_patch(
    State(state): State<AppState>,
    auth_ctx: AuthContext,
    headers: HeaderMap,
    Path(id): Path<i64>,
    body: String,
) -> Result<String, StatusCode> {
    todo!()
}
