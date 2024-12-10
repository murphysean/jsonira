use jsonwebtokens::{encode, Algorithm, Verifier};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    error::Error,
    time::{Duration, Instant, SystemTime, SystemTimeError, UNIX_EPOCH},
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("invalid data")]
    InvalidData,
    #[error("time issue")]
    TimeIssue,
    #[error("system time error")]
    SystemTimeError(#[from]SystemTimeError),
    #[error("jwt error")]
    JwtError(#[from]jsonwebtokens::error::Error),
    #[error("parse error")]
    ParseError,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct NewUser{
    pub email: String,
    pub password: String,
    pub name: String,
}

impl Into<User> for NewUser{
    fn into(self) -> User {
        User { id: 0, email: self.email, name: self.name, groups: None }
    }
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct User {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub email: String,

    #[serde(default)]
    pub name: String,

    //Auth Attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<String>>,
}

impl User {
    pub fn new() -> Self {
        Self::default()
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
            "sub": format!("{}", &self.id),
            "aud": "https://www.mrfy.io",
            "iat": now,
            "nbf": now,
            "exp": then,
            "jti": Uuid::new_v4(),
            "sid": Uuid::new_v4(),
            "name": &self.name,
            "email": &self.email,
        });
        Ok(encode(&header, &claims, &alg)?)
    }

    pub fn new_from_token(token: String, host: String) -> Result<User, UserError> {
        let alg = Algorithm::new_hmac(jsonwebtokens::AlgorithmID::HS256, "secret").unwrap();
        let verifier = Verifier::create()
            .issuer("https://www.mrfy.io")
            .audience(format!("https://{}", host))
            .build()?;
        let claims: Value = verifier.verify(&token, &alg)?;
        Ok(Self {
            id: claims
                .get("sub")
                .ok_or(UserError::InvalidData)?
                .as_str()
                .ok_or(UserError::InvalidData)?
                .parse()
                .map_err(|_|UserError::ParseError)?,
            email: claims
                .get("email")
                .ok_or(UserError::InvalidData)?
                .as_str()
                .ok_or(UserError::InvalidData)?
                .to_owned(),
            name: claims
                .get("name")
                .ok_or(UserError::InvalidData)?
                .as_str()
                .ok_or(UserError::InvalidData)?
                .to_owned(),
            groups: None,
        })
    }
}
