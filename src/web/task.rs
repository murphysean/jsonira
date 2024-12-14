
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_template::RenderHtml;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::{
    model::{
        subject::{AuthContext, Subject},
        task::Task,
    },
    AppState,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct ViewTaskData {
    subject: Subject,
    subject_json: String,
    task: Task,
    task_json: String,
}

#[tracing::instrument(level = "info")]
pub async fn view_task(
    State(state): State<AppState>,
    auth_ctx: AuthContext,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, (StatusCode,String)> {
    let key = "task";
    let engine = state.engine;


    match state.task_db.read_task(id).await {
        Ok(task) => {
            let data = ViewTaskData {
                subject: auth_ctx.subject.clone(),
                subject_json: to_string(&auth_ctx.subject).unwrap(),
                task: task.clone(),
                task_json: to_string(&task).unwrap(),
            };
            Ok(RenderHtml(key, engine, data))
        },
        Err(error) => Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string())),
    }
}
