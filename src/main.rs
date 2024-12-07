use chat_api::{user_connected, Users};
use session_api::{get_current_session, handle_post_login};
use std::sync::Arc;
use std::{collections::HashMap, env};
use todo_api::{todos_create, todos_delete, todos_list, todos_read, todos_update, Todo};
use tokio::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;
use user_api::{users_list, UserDb};
use warp::Filter;

mod chat_api;
mod session_api;
mod todo_api;
mod user_api;

/// A context that will be available at each handler
struct MyServerContext {
    token_secret: String,
    user_db: Arc<UserDb>,
    todo_db: Arc<Mutex<Vec<Todo>>>,
}

impl MyServerContext {
    pub fn new(secret_key: String) -> Self {
        Self {
            token_secret: secret_key,
            user_db: Arc::new(UserDb::new().unwrap()),
            todo_db: todo_api::blank_db(),
        }
    }
}

#[tokio::main]
async fn main() {
    let secret_key = env::var("SECRET_KEY").unwrap_or(String::from("secret"));
    let console_layer = console_subscriber::spawn();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(console_layer)
        .with(fmt_layer)
        .init();

    let context = MyServerContext::new(secret_key);

    let warp_users = {
        let db = context.user_db.clone();
        warp::path!("users")
            .and(warp::get())
            .and(warp::any().map(move || db.clone()))
            .and_then(users_list)
    };

    let todos = {
        let db = context.todo_db.clone();
        warp::path!("api" / "todos")
            .and(warp::get())
            .and(warp::any().map(move || db.clone()))
            .and(warp::query::<HashMap<String, String>>())
            .and_then(todos_list)
    }
    .or({
        let db = context.todo_db.clone();
        warp::path!("api" / "todos")
            .and(warp::post())
            .and(warp::any().map(move || db.clone()))
            .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json()))
            .and_then(todos_create)
    })
    .or({
        let db = context.todo_db.clone();
        warp::path!("api" / "todos" / usize)
            .and(warp::get())
            .and(warp::any().map(move || db.clone()))
            .and_then(todos_read)
    })
    .or({
        let db = context.todo_db.clone();
        warp::path!("api" / "todos" / usize)
            .and(warp::put())
            .and(warp::any().map(move || db.clone()))
            .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json()))
            .and_then(todos_update)
    })
    .or({
        let db = context.todo_db.clone();
        warp::path!("api" / "todos" / usize)
            .and(warp::delete())
            .and(warp::any().map(move || db.clone()))
            .and_then(todos_delete)
    });

    // Keep track of all connected users, key is usize, value
    // is a websocket sender.
    let users = Users::default();
    // Turn our "state" into a new Filter...
    let users = warp::any().map(move || users.clone());

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, users))
        });

    let session = {
        let db = context.user_db.clone();
        warp::path("session")
            .and(warp::get())
            .and(warp::any().map(move || db.clone()))
            .and(warp::filters::cookie::optional("session"))
            .and(warp::filters::header::header("host"))
            .and_then(get_current_session)
    };
    let login = {
        let db = context.user_db.clone();
        warp::path("login")
            .and(warp::post())
            .and(warp::body::form())
            .and(warp::any().map(move || db.clone()))
            .and_then(handle_post_login)
    };

    let routes = chat
        .or(login)
        .or(session)
        .or(warp_users)
        .or(todos)
        .or(warp::fs::dir("web"));

    warp::serve(routes)
    .tls()
    .cert_path("certs/fullchain.pem")
    .key_path("certs/privkey.pem")
    .run(([0, 0, 0, 0], 8443)).await;
}
