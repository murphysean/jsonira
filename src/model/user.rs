use eyre::Context;
use jsonwebtokens::{encode, Algorithm};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, Value};
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

use super::subject::Subject;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("invalid data")]
    InvalidData,
    #[error("time issue")]
    TimeIssue,
    #[error("system time error")]
    SystemTimeError(#[from] SystemTimeError),
    #[error("jwt error")]
    JwtError(#[from] jsonwebtokens::error::Error),
    #[error("parse error")]
    ParseError,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub name: String,
}

/// Attributes of a User
/// Based on JWT list found here: https://www.iana.org/assignments/jwt/jwt.xhtml
#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: Option<i64>,
    //Auth Attributes
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number_verified: Option<bool>,
    ///System managed groups
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entitlements: Option<Vec<String>>,
    ///User managed groups
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circles: Option<Vec<String>>,

    //Self managed attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthdate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
}

impl User {
    pub fn simple(id: i64, name: &str, email: &str) -> Self {
        Self {
            id: Some(id),
            email: Some(String::from(email)),
            name: Some(String::from(name)),
            ..Default::default()
        }
    }
}

impl From<NewUser> for User {
    fn from(value: NewUser) -> Self {
        Self {
            id: None,
            email: Some(value.email),
            email_verified: Some(false),
            phone_number_verified: Some(false),
            roles: None,
            groups: None,
            entitlements: None,
            circles: None,
            name: Some(value.name),
            profile: None,
            picture: None,
            website: None,
            gender: None,
            birthdate: None,
            phone_number: None,
        }
    }
}

impl TryFrom<Value> for User {
    type Error = eyre::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        //Need to do a special for the user id, which a jwt token requires be a string
        let id: Option<i64> = value
            .get("sub")
            .and_then(|v| v.as_str())
            .and_then(|v| v.parse::<i64>().ok())
            .or(value.get("id").and_then(|v| {
                let ret: Option<i64> = match v {
                    Value::Number(v) => v.as_i64(),
                    Value::String(s) => s.parse().ok(),
                    _ => None,
                };
                println!("Deep in here {:?}", ret);
                ret
            }));

        let result = from_value::<User>(value);
        match result {
            Ok(mut user) => {
                user.id = id;
                Ok(user)},
            Err(e) => Err(e.into()),
        }
    }
}

impl User {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn simplify(&self) -> Subject {
        Subject::User(User {
            id: self.id,
            email: self.email.clone(),
            name: self.name.clone(),
            ..Default::default()
        })
    }

    pub fn create_token(&self, secret: &str) -> Result<String, UserError> {
        let alg = Algorithm::new_hmac(jsonwebtokens::AlgorithmID::HS256, secret)?;
        let header = json!({"alg": alg.name()});
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let then = SystemTime::now()
            .checked_add(Duration::from_secs(60 * 60 * 8))
            .ok_or(UserError::TimeIssue)?
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let claims = json!({
            "iss": "https://www.mrfy.io",
            "sub": format!("{}", &self.id.unwrap_or_default()),
            "aud": "https://www.mrfy.io",
            "iat": now,
            "nbf": now,
            "exp": then,
            "jti": Uuid::new_v4(),
            "sid": Uuid::new_v4(),
            "name": &self.name,
            "email": &self.email,
            "circles": &self.circles,
        });
        Ok(encode(&header, &claims, &alg)?)
    }
}
