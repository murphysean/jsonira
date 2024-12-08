use axum::extract::Host;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::Form;
use axum::Json;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use crate::api::user::User;
use crate::MyServerContext;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimpleErr {
    pub message: String,
}

impl SimpleErr {
    fn new(message: String) -> Self {
        Self { message }
    }
}

pub async fn get_current_session(
    State(state): State<MyServerContext>,
    jar: CookieJar,
    Host(host): Host,
) -> Result<Json<User>, StatusCode> {
    let Some(cookie) = jar.get("session") else {
        return Err(StatusCode::NOT_FOUND);
    };
    let user = match User::new_from_token(cookie.value().to_string(), host) {
        Err(e) => {
            tracing::error!(e);
            return Err(StatusCode::UNAUTHORIZED);
        }
        Ok(user) => user,
    };
    if state.user_db.read_user(user.id).await.is_err() {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(user))
}

pub async fn handle_post_login(
    State(state): State<MyServerContext>,
    Form(form): Form<HashMap<String, String>>,
) -> Result<(HeaderMap, StatusCode), StatusCode> {
    let mut headers = HeaderMap::new();
    headers.insert("Location", "/index.html".parse().unwrap());
    if let Ok(user) = state
        .user_db
        .authenticate_user(form.get("username"), form.get("password"))
        .await
    {
        let token = user.create_token();
        headers.insert(
            "Set-Cookie",
            format!(
                "session={}; path=/; HttpOnly; SameSite=Strict; Secure",
                token
            )
            .parse()
            .unwrap(),
        );
    };

    Ok((headers, StatusCode::SEE_OTHER))
}
