use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use tracing::debug;
use warp::http::StatusCode;
use warp::reply;

use crate::user_api::User;
use crate::user_api::UserDb;

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
    users_database: Arc<UserDb>,
    token: Option<String>,
    host: String,
) -> Result<impl warp::Reply, Infallible> {
    let Some(token) = token else {
        return Ok(reply::with_status(
            reply::json(&SimpleErr::new(String::from("No Session, Please Login"))),
            StatusCode::NOT_FOUND,
        ));
    };
    let user = match User::new_from_token(token, host) {
        Err(e) => {
            tracing::error!(e);
            return Ok(reply::with_status(
                reply::json(&SimpleErr::new(String::from(
                    "Invalid Session, Please Login",
                ))),
                StatusCode::NOT_FOUND,
            ));
        }
        Ok(user) => user,
    };
    if users_database.read_user(user.id).await.is_err() {
        return Ok(reply::with_status(
            reply::json(&SimpleErr::new(String::from("Invalid User, Please Login"))),
            StatusCode::NOT_FOUND,
        ));
    }
    Ok(reply::with_status(reply::json(&user), StatusCode::OK))
}

pub async fn handle_post_login(
    form: HashMap<String, String>,
    users_database: Arc<UserDb>,
) -> Result<impl warp::Reply, Infallible> {
    let mut response = warp::http::Response::builder()
        .status(warp::http::StatusCode::SEE_OTHER)
        .header("Location", "/index.html");
    if let Ok(user) = users_database
        .authenticate_user(form.get("username"), form.get("password"))
        .await
    {
        let token = user.create_token();
        response = response.header(
            "Set-Cookie",
            format!(
                "session={}; path=/; HttpOnly; SameSite=Strict; Secure",
                token
            ),
        );
    };
    let response = response.body("").unwrap();

    Ok(response)
}
