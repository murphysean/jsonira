use axum::debug_handler;
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

use crate::model::subject::AuthContext;
use crate::model::subject::Subject;
use crate::model::user::User;

use super::ApiState;

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
    State(state): State<ApiState>,
    auth_ctx: AuthContext,
) -> Result<(HeaderMap, Json<User>), StatusCode> {
    println!("HERE 1");
    let Subject::User(user) = auth_ctx.subject else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    println!("HERE 2 {:?}", &user);
    let Some(id) = user.id else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    println!("HERE 3");
    let mut headers = HeaderMap::new();
    if let Ok(user) = state.user_db.read_user(id).await {
        let token = user.create_token(&state.token_secret).unwrap();
        println!("HERE 4");
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
    //Always update the session with a new cookie
    Ok((headers, Json(user)))
}

#[axum::debug_handler]
pub async fn handle_post_login(
    State(state): State<ApiState>,
    Form(form): Form<HashMap<String, String>>,
) -> Result<(HeaderMap, StatusCode), StatusCode> {
    let email = form.get("username").ok_or(StatusCode::BAD_REQUEST)?;
    let password = form.get("password").ok_or(StatusCode::BAD_REQUEST)?;
    let mut headers = HeaderMap::new();
    headers.insert("Location", "/index.html".parse().unwrap());
    if let Ok(user) = state.user_db.authenticate_user(email, password).await {
        let token = user.create_token(&state.token_secret).unwrap();
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
