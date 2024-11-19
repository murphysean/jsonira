use chat_api::{user_connected, Users};
use session_api::{get_current_session, handle_post_login};
use std::collections::HashMap;
use std::sync::Arc;
use todo_api::{todos_create, todos_delete, todos_list, todos_read, todos_update};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;
use user_api::{users_list, UserDb};
use warp::Filter;

mod chat_api;
mod session_api;
mod todo_api;
mod user_api;

#[tokio::main]
async fn main() {
    let console_layer = console_subscriber::spawn();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(console_layer)
        .with(fmt_layer)
        .init();

    let users_database = Arc::new(UserDb::new().unwrap());

    let db = todo_api::blank_db();

    let warp_users = {
        let db = users_database.clone();
        warp::path!("users")
            .and(warp::get())
            .and(warp::any().map(move || db.clone()))
            .and_then(users_list)
    };

    let todos = {
        let db = db.clone();
        warp::path!("api" / "todos")
            .and(warp::get())
            .and(warp::any().map(move || db.clone()))
            .and(warp::query::<HashMap<String, String>>())
            .and_then(todos_list)
    }
    .or({
        let db = db.clone();
        warp::path!("api" / "todos")
            .and(warp::post())
            .and(warp::any().map(move || db.clone()))
            .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json()))
            .and_then(todos_create)
    })
    .or({
        let db = db.clone();
        warp::path!("api" / "todos" / usize)
            .and(warp::get())
            .and(warp::any().map(move || db.clone()))
            .and_then(todos_read)
    })
    .or({
        let db = db.clone();
        warp::path!("api" / "todos" / usize)
            .and(warp::put())
            .and(warp::any().map(move || db.clone()))
            .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json()))
            .and_then(todos_update)
    })
    .or({
        let db = db.clone();
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
        let db = users_database.clone();
        warp::path("session")
            .and(warp::get())
            .and(warp::any().map(move || db.clone()))
            .and(warp::filters::cookie::optional("session"))
            .and(warp::filters::header::header("host"))
            .and_then(get_current_session)
    };
    let login = {
        let db = users_database.clone();
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

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
