use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::{extract::State, http::StatusCode, Json};
use json_patch::PatchOperation;
use serde_json::{from_value, json};
use tracing::{debug, error};

use crate::model::{
    task::{Action, Task, TaskPriority, TaskState},
    user::User,
};

use super::ApiState;

#[tracing::instrument(level = "info")]
pub async fn tasks_post(
    State(state): State<ApiState>,
    Json(task): Json<Task>,
) -> Result<Json<Task>, StatusCode> {
    //Need an authenticated user.
    let current_user: User = User::new();

    let now: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    //Insert the first history doc
    let patch: json_patch::Patch = from_value(json!([
      { "op": "add", "path": "/title", "value": "Dummy Task" },
      { "op": "add", "path": "/description", "value": "You need to be a dummy" },
      { "op": "add", "path": "/reporter", "value": current_user },
      { "op": "add", "path": "/priority", "value": "major" },
      { "op": "add", "path": "/estimate", "value": "1 hr" },
      { "op": "add", "path": "/points", "value": 10 },
      { "op": "add", "path": "/state", "value": TaskState::Todo },
      { "op": "add", "path": "/tags/-", "value": "2024-12-15" },
      { "op": "add", "path": "/created", "value": now },
      { "op": "add", "path": "/updated", "value": now },
      { "op": "add", "path": "/due", "value": now },
    ]))
    .unwrap();
    let action = Action {
        subject: current_user.clone(),
        patch: patch,
    };
    //Create the Task
    let task = Task {
        id: Some(0),
        title: String::from("Dummy Task"),
        description: String::from("You need to be a dummy"),
        reporter: Some(current_user),
        watchers: None,
        circle: None,
        assignee: None,
        priority: Some(TaskPriority::Major),
        estimate: Some(Duration::from_secs(60 * 60)),
        points: Some(10),
        state: TaskState::Todo,
        tags: vec![String::from("2024-12-15")],
        created: now,
        updated: now,
        due: Some(now),
        comments: vec![],
        reactions: vec![],
        reviews: vec![],
        history: vec![action],
    };

    Ok(Json(task))
}

pub async fn task_get(State(state): State<ApiState>) -> Result<Json<Task>, StatusCode> {
    todo!()
}
