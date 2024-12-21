use axum::extract::State;
use axum::http::uri::Scheme;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::http::Uri;
use axum::Form;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

use crate::model::subject::AuthContext;
use crate::model::subject::Subject;
use crate::model::user::User;

use crate::AppState;

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
    State(state): State<AppState>,
    //uri: Uri,
    auth_ctx: AuthContext,
) -> Result<(HeaderMap, Json<User>), StatusCode> {
    let mut secure_attr = "";
    //if uri.scheme() == Some(&Scheme::HTTPS){
    //    secure_attr = "; Secure";
    //}
    let Subject::User(user) = auth_ctx.subject else {
        println!("Huh...");
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Some(id) = user.id else {
        println!("oh...");
        return Err(StatusCode::UNAUTHORIZED);
    };
    println!("interesting...");
    let mut headers = HeaderMap::new();
    if let Ok(user) = state.user_db.read_user(id).await {
        let token = user.create_token(&state.token_secret).unwrap();
        headers.insert(
            "Set-Cookie",
            format!(
                "session={}; path=/; HttpOnly; SameSite=Strict{}",
                token,
                secure_attr
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
    State(state): State<AppState>,
    uri: Uri,
    Form(form): Form<HashMap<String, String>>,
) -> Result<(HeaderMap, StatusCode), StatusCode> {
    println!("Scheme {:?}", uri.scheme());
    let mut secure_attr = "";
    if uri.scheme() == Some(&Scheme::HTTPS){
        secure_attr = "; Secure";
    }
    let email = form.get("username").ok_or(StatusCode::BAD_REQUEST)?;
    let password = form.get("password").ok_or(StatusCode::BAD_REQUEST)?;
    let mut headers = HeaderMap::new();
    headers.insert("Location", "/index.html".parse().unwrap());
    match state.user_db.authenticate_user(email, password).await {
        Ok(user) => {
            let token = user.create_token(&state.token_secret).unwrap();
            headers.insert(
                "Set-Cookie",
                format!(
                    "session={}; path=/; HttpOnly; SameSite=Strict{}",
                    token,
                    secure_attr
                )
                .parse()
                .unwrap(),
            );
        }
        Err(e) => {
            println!("Failed login: {}",e);
            Err(StatusCode::BAD_REQUEST)?;
        }
    }

    Ok((headers, StatusCode::SEE_OTHER))
}
