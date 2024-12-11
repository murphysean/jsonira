use std::{default, result};

use axum::{
    async_trait,
    extract::{FromRequestParts, Host, OriginalUri, Path, State},
    http::{request::Parts, Method, StatusCode, Uri},
    response::{IntoResponse, IntoResponseParts},
};
use axum_extra::extract::CookieJar;
use jsonwebtokens::{Algorithm, Verifier};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::api::ApiState;

use super::user::User;

/// Subject is meant to be used as a reference to other users or clients
/// It is also used to represent the currently authenticated user or client at the handler
#[derive(Default, Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum Subject {
    #[default]
    None,
    UserId(i64),
    UserEmail(String),
    User(User),
    Client(Client),
}

impl PartialEq for Subject {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            Subject::None => false,
            Subject::UserId(id) => match &other {
                Subject::UserId(oid) => id == oid,
                Subject::User(other) => other.id.map(|oid| *id == oid).unwrap_or(false),
                _ => false,
            },
            Subject::UserEmail(email) => match &other {
                Subject::UserEmail(oemail) => email == oemail,
                Subject::User(other) => other
                    .email
                    .as_ref()
                    .map(|oemail| email == oemail)
                    .unwrap_or(false),
                _ => false,
            },
            Subject::User(user) => match &other {
                Subject::UserId(oid) => user.id.map(|id| id == *oid).unwrap_or(false),
                Subject::UserEmail(oemail) => user
                    .email
                    .as_ref()
                    .map(|email| email == oemail)
                    .unwrap_or(false),
                Subject::User(other) => {
                    user.id
                        .map(|id| other.id.map(|oid| id == oid).unwrap_or(false))
                        .unwrap_or(false)
                        || user
                            .email
                            .as_ref()
                            .map(|email| {
                                other
                                    .email
                                    .as_ref()
                                    .map(|oemail| email == oemail)
                                    .unwrap_or(false)
                            })
                            .unwrap_or(false)
                }
                _ => false,
            },
            Subject::Client(client) => match other {
                Subject::Client(other) => client.client_id == other.client_id,
                _ => false,
            },
        }
    }
}

impl TryFrom<Value> for Subject {
    type Error = eyre::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match &value {
            Value::Number(num) => Ok(Subject::UserId(
                num.as_i64().ok_or(eyre::eyre!("Number must by i64"))?,
            )),
            Value::String(string) => Ok(Subject::UserEmail(string.clone())),
            //TODO Subject will be the client variant iff client_id = sub
            Value::Object(_) => Ok(Subject::User(value.try_into()?)),
            _ => Ok(Subject::None),
        }
    }
}

impl From<User> for Subject {
    fn from(value: User) -> Self {
        Self::User(value)
    }
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct Client {
    pub client_id: String,
}

impl TryFrom<&Value> for Client {
    type Error = eyre::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct AuthContext {
    pub subject: Subject,
    pub client: Client,
    pub environment: Environment,
    pub action: Action,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct Environment {
    pub issuer: String,
    pub audience: String,
    pub jwt_id: String,
    pub session_id: String,
    pub host: String,
    pub token: String,
}

impl TryFrom<Value> for Environment {
    type Error = eyre::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(Self {
            issuer: value
                .get("iss")
                .and_then(|v| v.as_str())
                .ok_or(eyre::eyre!("Invalid Issuer"))?
                .to_owned(),
            audience: value
                .get("aud")
                .and_then(|v| v.as_str())
                .ok_or(eyre::eyre!("Invalid Issuer"))?
                .to_owned(),
            jwt_id: value
                .get("jti")
                .and_then(|v| v.as_str())
                .ok_or(eyre::eyre!("Invalid Issuer"))?
                .to_owned(),
            session_id: value
                .get("sid")
                .and_then(|v| v.as_str())
                .ok_or(eyre::eyre!("Invalid Issuer"))?
                .to_owned(),
            host: String::default(),
            token: String::default(),
        })
    }
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct Action {
    pub scheme: String,
    pub authority: String,
    pub path_and_query: String,
    pub method: String,
}

#[derive(Debug, Error)]
pub enum SubjectRejection {
    #[error("failed to resolve host")]
    FailedToResolveHost,
    #[error("failed to parse path")]
    FailedToParseUri,
    #[error("invalid input")]
    NoSubject,
    #[error("jwt error")]
    JsonWebTokenError(#[from] jsonwebtokens::error::Error),
    #[error("serde error")]
    SerdeError(#[from] eyre::Error),
}

impl IntoResponse for SubjectRejection {
    fn into_response(self) -> axum::response::Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = SubjectRejection;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let result = Host::from_request_parts(parts, state).await;
        let Ok(Host(host)) = result else {
            return Err(SubjectRejection::FailedToResolveHost);
        };
        let Ok(OriginalUri(uri)) = OriginalUri::from_request_parts(parts, state).await;
        //TODO See if I can pull the api state off of the request
        //let State(state) : State<ApiContext> = State::from_request_parts(parts, state).await.unwrap();
        //Pull session cookie
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        if let Some(cookie) = jar.get("session") {
            let token = cookie.value().to_string();
            let alg = Algorithm::new_hmac(jsonwebtokens::AlgorithmID::HS256, "secret")?;
            let verifier = Verifier::create()
                .issuer("https://www.mrfy.io")
                .audience(format!("https://{}", host))
                .build()?;
            let claims: Value = verifier.verify(&token, &alg)?;
            let ctx = AuthContext {
                subject: claims.clone().try_into()?,
                client: Client {
                    client_id: String::from("mrfy.io"),
                },
                environment: claims.try_into()?,
                action: Action {
                    scheme: uri
                        .scheme()
                        .ok_or(SubjectRejection::FailedToParseUri)?
                        .as_str()
                        .to_owned(),
                    authority: uri
                        .authority()
                        .ok_or(SubjectRejection::FailedToParseUri)?
                        .as_str()
                        .to_owned(),
                    path_and_query: uri
                        .path_and_query()
                        .ok_or(SubjectRejection::FailedToParseUri)?
                        .as_str()
                        .to_owned(),
                    method: parts.method.as_str().to_owned(),
                },
            };
            return Ok(ctx);
        }

        //TODO Pull Authorization Bearer token
        Err(SubjectRejection::NoSubject)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, Value};

    use crate::model::{subject::Subject, user::User};

    #[test]
    fn test_subject_serialization() {
        let simple_email: Subject = from_str("\"murphysean84@gmail.com\"").unwrap();
        println!("Simple Email: {:?}", simple_email);
        assert_eq!(
            simple_email,
            Subject::UserEmail(String::from("murphysean84@gmail.com"))
        );
    }

    #[test]
    fn test_subject_equals() {
        assert_eq!(
            Subject::UserId(20), 
            Subject::UserId(20)
        );
        // Tokens (JWT) have a sub typed as a str
        let token: Value = from_str(r#"{
            "sub": "20",
            "email": "murphysean84@gmail.com"
        }"#).unwrap();
        let token_user: User = token.try_into().unwrap();
        let simple_user: User = from_str(r#"{
            "id": 20,
            "email": "murphysean84@gmail.com"
        }"#).unwrap();
        assert_eq!(
            Subject::UserId(20), 
            Subject::User(token_user.clone()),
        );
        assert_eq!(
            Subject::UserId(20), 
            Subject::User(simple_user.clone()),
        );
        assert_eq!(
            Subject::UserEmail(String::from("murphysean84@gmail.com")), 
            Subject::User(simple_user),
        );
    }
}
