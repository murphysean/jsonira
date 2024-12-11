use std::{fmt::Debug, sync::Arc};

use todo::{blank_db, Todo};
use tokio::sync::Mutex;

use crate::db::{chat::ChatDb, user::UserDb};

pub mod chat;
pub mod event;
pub mod session;
pub mod task;
pub mod todo;
pub mod user;

#[derive(Clone)]
pub struct ApiState {
    token_secret: String,
    user_db: Arc<UserDb>,
    chat_db: Arc<ChatDb>,
    todo_db: Arc<Mutex<Vec<Todo>>>,
    //task_db: Arc<TaskDb>,
}

impl Debug for ApiState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiContext").finish()
    }
}

impl ApiState {
    pub fn new(secret_key: String) -> Self {
        Self {
            token_secret: secret_key,
            user_db: Arc::new(UserDb::new().unwrap()),
            chat_db: Arc::new(ChatDb::new().unwrap()),
            todo_db: blank_db(),
        }
    }
}
