use axum::extract::Host;
use axum::handler::HandlerWithoutStateExt;
use axum::http::{StatusCode, Uri};
use axum::response::Redirect;
use axum::routing::method_routing::{delete, get, options, patch, post, put};
use axum::routing::IntoMakeService;
use axum::{BoxError, Router};
use axum_server::tls_rustls::RustlsConfig;
use session_api::{get_current_session, handle_post_login};
use std::net::SocketAddr;
use std::sync::Arc;
use std::{collections::HashMap, env};
use todo_api::{todos_create, todos_delete, todos_list, todos_read, todos_update, Todo};
use tokio::sync::Mutex;
use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;
use user_api::{users_list, UserDb};

mod session_api;
mod todo_api;
mod user_api;

/// A context that will be available at each handler
#[derive(Clone)]
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

    tokio::spawn(redirect_http_to_https());

    let app = Router::new()
        .route("/users", get(users_list))
        .route("/api/todos", get(todos_list))
        //.route("/api/todos", post(todos_create))
        .route("/api/todos/{id}", get(todos_read))
        //.route("/api/todos/{id}", put(todos_update))
        .route("/api/todos/{id}", delete(todos_delete))
        .route("/login", post(handle_post_login))
        .route("/session", get(get_current_session))
        .nest_service("/", ServeDir::new("web"))
        .with_state(context);

    // configure certificate and private key used by https
    let config = RustlsConfig::from_pem_file("certs/fullchain.pem", "certs/privkey.pem")
        .await
        .unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn redirect_http_to_https() {
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
        .await
        .unwrap();
}
