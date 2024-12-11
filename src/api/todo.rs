use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{Path, Query, State},
    http::{Response, StatusCode},
    Json,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::to_string;
use tokio::sync::Mutex;

use super::AppState;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimpleErr {
    pub message: String,
}

impl SimpleErr {
    fn new(message: String) -> Self {
        Self { message }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Todo {
    pub id: usize,
    pub created_by: String,
    pub create_date: u64,
    pub update_date: u64,
    pub complete: bool,
    pub text: String,
    pub deleted: bool,
}

impl Todo {
    pub fn new_from(id: usize, other: &Self) -> Self {
        Self {
            id,
            created_by: other.created_by.to_owned(),
            create_date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            update_date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            complete: false,
            text: other.text.to_owned(),
            deleted: false,
        }
    }

    pub fn mark_deleted(&mut self) {
        self.created_by = String::from("deleted");
        self.update_date = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.deleted = true;
    }

    pub fn update_from(&mut self, other: &Self) {
        if self.deleted {
            self.create_date = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.created_by = other.created_by.to_owned();
        }
        self.update_date = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.complete = other.complete;
        self.text = other.text.to_owned();
        self.deleted = false;
    }
}

pub type TodoDb = Arc<Mutex<Vec<Todo>>>;

pub fn blank_db() -> TodoDb {
    Arc::new(Mutex::new(Vec::new()))
}

pub async fn todos_list(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Todo>>, StatusCode> {
    let offset: Option<usize> = query.get("offset").and_then(|offset| offset.parse().ok());
    let limit: Option<usize> = query.get("limit").and_then(|limit| limit.parse().ok());

    let todos = state.todo_db.lock().await;
    let todos: Vec<Todo> = todos
        .clone()
        .into_iter()
        .skip(offset.unwrap_or_default())
        .filter(|x| x.deleted)
        .take(limit.unwrap_or(10))
        .collect();
    Ok(Json(todos))
}

#[axum::debug_handler]
pub async fn todos_create(
    State(state): State<AppState>,
    Json(todo): Json<Todo>,
) -> Result<Response<String>, StatusCode> {
    let mut todos = state.todo_db.lock().await;
    let id = todos.len();
    let new_todo = Todo::new_from(id, &todo);
    let body = to_string(&new_todo).unwrap();

    todos.push(new_todo);

    let response = Response::builder()
        .status(StatusCode::CREATED)
        .header("Location", format!("todos/{}", id))
        .body(body)
        .unwrap();

    Ok(response)
}

pub async fn todos_read(
    State(state): State<AppState>,
    Path(id): Path<usize>,
) -> Result<Json<Todo>, StatusCode> {
    let todos = state.todo_db.lock().await;
    let Some(todo) = todos.get(id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    if todo.deleted {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(todo.clone()))
}

pub async fn todos_update(
    State(state): State<AppState>,
    Path(id): Path<usize>,
    Json(todo): Json<Todo>,
) -> Result<Response<String>, StatusCode> {
    let mut todos = state.todo_db.lock().await;
    let Some(db_todo) = todos.get_mut(id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let mut response = Response::builder();
    if db_todo.deleted {
        response = response.status(StatusCode::CREATED)
    } else {
        response = response.status(StatusCode::OK)
    }
    let body = to_string(&db_todo).unwrap();
    db_todo.update_from(&todo);

    Ok(response.body(body).unwrap())
}

pub async fn todos_delete(
    State(state): State<AppState>,
    Path(id): Path<usize>,
) -> Result<StatusCode, StatusCode> {
    let mut todos = state.todo_db.lock().await;
    let Some(db_todo) = todos.get_mut(id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    db_todo.mark_deleted();

    Ok(StatusCode::NO_CONTENT)
}
