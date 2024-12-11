use api::chat::{
    chats_get_room, chats_get_room_messages, chats_get_rooms, chats_post_room,
    chats_post_room_message,
};
use api::event::server_sent_events;
use api::session::{get_current_session, handle_post_login};
use api::task::tasks_post;
use api::todo::{todos_create, todos_delete, todos_list, todos_read, todos_update};
use api::user::{user_delete, user_get, user_patch, user_put, users_get, users_post};
use api::AppState;
use axum::extract::Host;
use axum::handler::HandlerWithoutStateExt;
use axum::http::{StatusCode, Uri};
use axum::response::Redirect;
use axum::routing::method_routing::{delete, get, post, put};
use axum::routing::patch;
use axum::{BoxError, Router, ServiceExt};
use axum_server::tls_rustls::RustlsConfig;
use std::env;
use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::signal;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;

mod api;
mod db;
mod model;
mod web;

#[tokio::main]
async fn main() {
    let secret_key = env::var("SECRET_KEY").unwrap_or(String::from("secret"));
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

    let context = AppState::new(secret_key);

    // Create a handle for our tls server so the shutdown signal can all shutdown
    let handle = axum_server::Handle::new();
    // save the future for easy shutting down of our redirect server
    let shutdown_future = shutdown_signal(handle.clone());

    tokio::spawn(redirect_http_to_https(shutdown_future));

    let app = Router::new()
        .route("/api/users", get(users_get))
        .route("/api/users", post(users_post))
        .route("/api/users/:id", get(user_get))
        .route("/api/users/:id", put(user_put))
        .route("/api/users/:id", patch(user_patch))
        .route("/api/users/:id", delete(user_delete))
        .route("/api/tasks", post(tasks_post))
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
        .fallback_service(ServeDir::new("web"))
        .layer(TraceLayer::new_for_http())
        .with_state(context);

    // configure certificate and private key used by https
    let config = RustlsConfig::from_pem_file("certs/fullchain.pem", "certs/privkey.pem")
        .await
        .unwrap();

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
