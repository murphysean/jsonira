use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use crate::user_api::UserDb;

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
