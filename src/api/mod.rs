use std::{fmt::Debug, sync::Arc};

use todo::{blank_db, Todo};
use tokio::sync::Mutex;

use crate::db::{chat::ChatDb, user::UserDb};

pub mod abac;
pub mod chat;
pub mod event;
pub mod session;
pub mod task;
pub mod todo;
pub mod user;

#[derive(Clone)]
pub struct AppState {
    pub token_secret: String,
    pub user_db: Arc<UserDb>,
    pub chat_db: Arc<ChatDb>,
    pub todo_db: Arc<Mutex<Vec<Todo>>>,
    //task_db: Arc<TaskDb>,
}

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiContext").finish()
    }
}

impl AppState {
    pub fn new(secret_key: String) -> Self {
        Self {
            token_secret: secret_key,
            user_db: Arc::new(UserDb::new().unwrap()),
            chat_db: Arc::new(ChatDb::new().unwrap()),
            todo_db: blank_db(),
        }
    }
}
