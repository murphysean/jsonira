use std::{collections::HashMap, convert::Infallible, sync::Arc};

use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use tokio::sync::Mutex;

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
    pub create_date: DateTime<Utc>,
    pub update_date: DateTime<Utc>,
    pub complete: bool,
    pub text: String,
    pub deleted: bool,
}

impl Todo {
    pub fn new_from(id: usize, other: &Self) -> Self {
        Self {
            id,
            created_by: other.created_by.to_owned(),
            create_date: Utc::now(),
            update_date: Utc::now(),
            complete: false,
            text: other.text.to_owned(),
            deleted: false,
        }
    }

    pub fn mark_deleted(&mut self) {
        self.created_by = String::from("deleted");
        self.update_date = Utc::now();
        self.deleted = true;
    }

    pub fn update_from(&mut self, other: &Self) {
        if self.deleted {
            self.create_date = Utc::now();
            self.created_by = other.created_by.to_owned();
        }
        self.update_date = Utc::now();
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
    db: TodoDb,
    query: HashMap<String, String>,
) -> Result<impl warp::Reply, Infallible> {
    let offset: Option<usize> = query.get("offset").and_then(|offset| offset.parse().ok());
    let limit: Option<usize> = query.get("limit").and_then(|limit| limit.parse().ok());

    let todos = db.lock().await;
    let todos: Vec<Todo> = todos
        .clone()
        .into_iter()
        .skip(offset.unwrap_or_default())
        .filter(|x| x.deleted)
        .take(limit.unwrap_or(10))
        .collect();
    Ok(warp::reply::json(&todos))
}

pub async fn todos_create(db: TodoDb, todo: Todo) -> Result<impl warp::Reply, Infallible> {
    let mut todos = db.lock().await;
    let id = todos.len();
    let todo = Todo::new_from(id, &todo);

    let reply = warp::reply::with_status(
        warp::reply::with_header(
            warp::reply::json(&todo),
            "Location",
            format!("todos/{}", id),
        ),
        warp::http::StatusCode::CREATED,
    );
    todos.push(todo);

    Ok(reply)
}

pub async fn todos_read(id: usize, db: TodoDb) -> Result<impl warp::Reply, Infallible> {
    let todos = db.lock().await;
    let Some(todo) = todos.get(id) else {
        return Ok(warp::reply::with_status(
            warp::reply::json(&SimpleErr::new(String::from("Not Found"))),
            warp::http::StatusCode::NOT_FOUND,
        ));
    };
    if todo.deleted {
        return Ok(warp::reply::with_status(
            warp::reply::json(&SimpleErr::new(String::from("Not Found"))),
            warp::http::StatusCode::NOT_FOUND,
        ));
    }
    Ok(warp::reply::with_status(
        warp::reply::json(&todo),
        warp::http::StatusCode::OK,
    ))
}

pub async fn todos_update(
    id: usize,
    db: TodoDb,
    todo: Todo,
) -> Result<impl warp::Reply, Infallible> {
    let mut todos = db.lock().await;
    let Some(db_todo) = todos.get_mut(id) else {
        return Ok(warp::reply::with_status(
            warp::reply::json(&SimpleErr::new(String::from("Not Found"))),
            warp::http::StatusCode::NOT_FOUND,
        ));
    };
    let mut return_status = warp::http::StatusCode::OK;
    if db_todo.deleted {
        return_status = warp::http::StatusCode::CREATED;
    }
    db_todo.update_from(&todo);

    Ok(warp::reply::with_status(
        warp::reply::json(&db_todo),
        return_status,
    ))
}

pub async fn todos_delete(id: usize, db: TodoDb) -> Result<impl warp::Reply, Infallible> {
    let mut todos = db.lock().await;
    let Some(db_todo) = todos.get_mut(id) else {
        return Ok(warp::http::StatusCode::NOT_FOUND);
    };
    db_todo.mark_deleted();

    Ok(warp::http::StatusCode::NO_CONTENT)
}
