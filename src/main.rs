use api::chat::{
    chats_get_room, chats_get_room_messages, chats_get_rooms, chats_post_room,
    chats_post_room_message,
};
use api::event::server_sent_events;
use api::session::{get_current_session, handle_post_login};
use api::task::{task_get, task_patch, tasks_get, tasks_post};
use api::todo::{blank_db, Todo};
use api::todo::{todos_create, todos_delete, todos_list, todos_read, todos_update};
use api::user::{user_delete, user_get, user_patch, user_put, users_get, users_post};
use axum::extract::Host;
use axum::handler::HandlerWithoutStateExt;
use axum::http::{StatusCode, Uri};
use axum::response::Redirect;
use axum::routing::method_routing::{delete, get, post, put};
use axum::routing::patch;
use axum::{BoxError, Router, ServiceExt};
use axum_server::tls_rustls::RustlsConfig;
use axum_template::engine::Engine;
use db::task::TaskDb;
use db::{chat::ChatDb, user::UserDb};
use handlebars::{DirectorySourceOptions, Handlebars};
use std::env;
use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;
use std::{fmt::Debug, sync::Arc};
use tokio::signal;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;
use web::task::view_task;

mod api;
mod db;
mod model;
mod web;

#[derive(Clone)]
pub struct AppState {
    pub token_secret: String,
    pub user_db: Arc<UserDb>,
    pub task_db: Arc<TaskDb>,
    pub chat_db: Arc<ChatDb>,
    pub todo_db: Arc<Mutex<Vec<Todo>>>,
    //task_db: Arc<TaskDb>,
    pub engine: Engine<Handlebars<'static>>,
}

impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiContext").finish()
    }
}

impl AppState {
    pub fn new() -> Self {
        let secret_key = env::var("SECRET_KEY").unwrap_or(String::from("secret"));
        let mut hbs = Handlebars::new();
        hbs.register_templates_directory("web/templates", DirectorySourceOptions::default())
            .unwrap();
        hbs.set_dev_mode(true);

        Self {
            token_secret: secret_key,
            user_db: Arc::new(UserDb::new().unwrap()),
            task_db: Arc::new(TaskDb::new().unwrap()),
            chat_db: Arc::new(ChatDb::new().unwrap()),
            todo_db: blank_db(),
            engine: Engine::from(hbs),
        }
    }
}

#[tokio::main]
async fn main() {
    let console_layer = console_subscriber::spawn();
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // axum logs rejections from built-in extractors with the `axum::rejection`
        // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
        format!(
            "{}=debug,tower_http=debug,axum::rejection=trace",
            env!("CARGO_CRATE_NAME")
        )
        .into()
    });
    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(env_filter);
    //.with_filter(LevelFilter::TRACE);
    tracing_subscriber::registry()
        .with(console_layer)
        .with(fmt_layer)
        .init();

    let context = AppState::new();

    // Create a handle for our tls server so the shutdown signal can all shutdown
    let handle = axum_server::Handle::new();
    // save the future for easy shutting down of our redirect server
    let shutdown_future = shutdown_signal(handle.clone());

    //tokio::spawn(redirect_http_to_https(shutdown_future));

    let app = Router::new()
        .route("/api/users", get(users_get))
        .route("/api/users", post(users_post))
        .route("/api/users/:id", get(user_get))
        .route("/api/users/:id", put(user_put))
        .route("/api/users/:id", patch(user_patch))
        .route("/api/users/:id", delete(user_delete))
        .route("/api/tasks", get(tasks_get))
        .route("/api/tasks", post(tasks_post))
        .route("/api/tasks/:id", get(task_get))
        .route("/api/tasks/:id", patch(task_patch))
        .route("/api/todos", get(todos_list))
        .route("/api/todos", post(todos_create))
        .route("/api/todos/:id", get(todos_read))
        .route("/api/todos/:id", put(todos_update))
        .route("/api/todos/:id", delete(todos_delete))
        .route("/api/chat/rooms", get(chats_get_rooms))
        .route("/api/chat/rooms", post(chats_post_room))
        .route("/api/chat/rooms/:id", get(chats_get_room))
        .route("/api/chat/rooms/:id/messages", get(chats_get_room_messages))
        .route(
            "/api/chat/rooms/:id/messages",
            post(chats_post_room_message),
        )
        .route("/api/events", get(server_sent_events))
        .route("/login", post(handle_post_login))
        .route("/session", get(get_current_session))
        .route("/task/:id", get(view_task))
        .fallback_service(ServeDir::new("web"))
        .layer(TraceLayer::new_for_http())
        .with_state(context);

    // configure certificate and private key used by https
    let config = RustlsConfig::from_pem_file("certs/fullchain.pem", "certs/privkey.pem")
        .await
        .unwrap();

    let appc = app.clone();
    let handlec = handle.clone();
    tokio::spawn(async move{
        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        axum_server::bind(addr)
        .handle(handlec)
        .serve(appc.into_make_service())
        .await
        .unwrap();
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn shutdown_signal(handle: axum_server::Handle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Received termination signal shutting down");
    handle.graceful_shutdown(Some(Duration::from_secs(10))); // 10 secs is how long docker will wait
                                                             // to force shutdown
}

async fn redirect_http_to_https<F>(signal: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    fn make_https(host: String, uri: Uri) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&"80".to_string(), &"443".to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, redirect.into_make_service())
        .with_graceful_shutdown(signal)
        .await
        .unwrap();
}
