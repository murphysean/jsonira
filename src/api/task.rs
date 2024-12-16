use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    Json,
};
use axum_extra::extract::Query;
use jiff::Timestamp;
use json_patch::Patch;
use serde::Deserialize;
use serde_json::{from_str, from_value, to_value, Value};
use tracing::{debug, error};

use crate::model::{
    subject::{AuthContext, Subject},
    task::{Task, TaskState},
};

use crate::AppState;

use super::abac::Decision;

#[derive(Debug, Deserialize)]
pub struct TasksQueryParams {
    #[serde(default)]
    pub tag: Vec<String>,
    #[serde(default)]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[tracing::instrument(level = "info")]
pub async fn tasks_get(
    State(state): State<AppState>,
    auth_ctx: AuthContext,
    Query(query): Query<TasksQueryParams>,
) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    let mut limit = query.limit;
    if limit == 0 {
        limit = 100
    };
    let Subject::User(user) = auth_ctx.subject else {
        return Err((
            StatusCode::FORBIDDEN,
            String::from("Authenticated Subject is not a user"),
        ));
    };

    println!("circles: {:?}", user.circles);
    println!("Tags: {:?}", query.tag);

    match state
        .task_db
        .read_tasks(limit, query.offset, user.circles, Some(query.tag))
        .await
    {
        Ok(users) => Ok(Json(users)),
        Err(error) => {
            error!("{:?}", error);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string()));
        }
    }
}

#[tracing::instrument(level = "info")]
pub async fn tasks_post(
    State(state): State<AppState>,
    auth_ctx: AuthContext,
    Json(mut task): Json<Task>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let Subject::User(user) = &auth_ctx.subject else {
        return Err((
            StatusCode::FORBIDDEN,
            String::from("Authenticated Subject is not a user"),
        ));
    };
    //Anyone can create a task,
    //but you can't set the circle to something you aren't in
    //and you
    auth_ctx
        .enforce_policy(|_actx| Decision::Permit)
        .map_err(|_| ((StatusCode::FORBIDDEN, String::from("Policy Forbids"))))?;
    let user = user.simplify();

    let now: Timestamp = Timestamp::now();
    //Set non-negotiables on task
    task.id = None;
    task.reporter = Some(user.clone());
    task.state = Some(TaskState::Todo);
    task.created = Some(now);
    task.updated = Some(now);

    //Initialize if empty
    if task.watchers.is_none() {
        task.watchers = Some(vec![]);
    }
    if task.tags.is_none() {
        task.tags = Some(vec![]);
    }
    if task.comments.is_none() {
        task.comments = Some(vec![]);
    }
    if task.reactions.is_none() {
        task.reactions = Some(vec![]);
    }
    if task.reviews.is_none() {
        task.reviews = Some(vec![]);
    }

    let action = task.generate_action(user.clone());
    task.history = Some(vec![action]);

    //Store it
    match state.task_db.create_task(task).await {
        Ok(task) => Ok(Json(task)),
        Err(error) => return Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string())),
    }
}

pub async fn task_get(
    State(state): State<AppState>,
    auth_ctx: AuthContext,
    Path(id): Path<i64>,
) -> Result<Json<Task>, (StatusCode, String)> {
    match state.task_db.read_task(id).await {
        Ok(task) => Ok(Json(task)),
        Err(error) => return Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string())),
    }
}

pub async fn task_patch(
    State(state): State<AppState>,
    auth_ctx: AuthContext,
    headers: HeaderMap,
    Path(id): Path<i64>,
    body: String,
) -> Result<Json<Task>, (StatusCode, String)> {
    let Subject::User(user) = &auth_ctx.subject else {
        return Err((
            StatusCode::FORBIDDEN,
            String::from("Authenticated Subject is not a user"),
        ));
    };
    let task = match state.task_db.read_task(id).await {
        Ok(task) => task,
        Err(error) => Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?,
    };

    let mut doc: Value = to_value(&task).map_err(|e| {
        error!(error= %e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    //Check the content type, merge strategy depends on
    match headers.get("content-type") {
        Some(hv) if hv == HeaderValue::from_static("application/json-patch+json") => {
            let patch: Patch = from_str(&body).map_err(|e| {
                error!(error= %e);
                (StatusCode::BAD_REQUEST, e.to_string())
            })?;
            //TODO Do buisness logic on the patch set
            task.policy(&auth_ctx, &patch);
            json_patch::patch(&mut doc, &patch).map_err(|e| {
                error!(error= %e);
                (StatusCode::BAD_REQUEST, e.to_string())
            })?;
            //TODO Create an Action wrapping this patch and append it to the task history
        }
        Some(hv) if hv == HeaderValue::from_static("application/json") => {
            let patch: Value = from_str(&body).map_err(|e| {
                error!(error= %e);
                (StatusCode::BAD_REQUEST, e.to_string())
            })?;
            let proposed_task: Task = from_value(patch.clone()).map_err(|e| {
                error!(error= %e);
                (StatusCode::BAD_REQUEST, e.to_string())
            })?;
            let proposed_action = proposed_task.generate_action(auth_ctx.subject.clone());
            //TODO Do buisness logic on the patch set
            task.policy(&auth_ctx, &proposed_action.patch);
            json_patch::merge(&mut doc, &patch);
            //TODO Create an Action wrapping this patch and append it to the task history
        }
        _ => {
            return Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                String::from("Unsupported media type"),
            ))
        }
    }
    let mut task: Task = from_value(doc).map_err(|e| {
        error!(error= %e);
        (StatusCode::BAD_REQUEST, e.to_string())
    })?;
    task.id = Some(id);
    let result = state.task_db.update_task(id, task).await;
    match result.inspect_err(|e| debug!("Error: {}", e)) {
        Ok(user) => Ok(Json(user)),
        Err(error) => Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string())),
    }
}
