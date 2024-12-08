use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{Duration, Utc};
use jsonwebtokens::{encode, Algorithm, Verifier};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::fmt;
use std::io::Error as IoError;
use std::sync::Arc;
use std::{error::Error, io::ErrorKind};
use tokio::sync::Mutex;
use uuid::Uuid;

use sqlite::Statement;

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

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub name: Option<String>,

    //Auth Attributes
    pub groups: Option<Vec<String>>,
}

impl User {
    pub fn new() -> Self {
        Self::default()
    }

    fn new_from_row(statement: &mut Statement) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            id: statement.read("id")?,
            email: statement.read("email")?,
            name: statement.read("name")?,
            groups: None,
        })
    }

    pub fn create_token(&self) -> String {
        let alg = Algorithm::new_hmac(jsonwebtokens::AlgorithmID::HS256, "secret").unwrap();
        let header = json!({"alg": alg.name()});
        let then = Utc::now() + Duration::hours(8);
        let claims = json!({
            "iss": "https://www.mrfy.io",
            "sub": format!("{}", &self.id),
            "aud": "https://www.mrfy.io",
            "iat": Utc::now().timestamp(),
            "nbf": Utc::now().timestamp(),
            "exp": then.timestamp(),
            "jti": Uuid::new_v4(),
            "sid": Uuid::new_v4(),
            "name": &self.name,
            "email": &self.email,
        });
        encode(&header, &claims, &alg).unwrap()
    }

    pub fn new_from_token(token: String, host: String) -> Result<User, Box<dyn Error>> {
        let alg = Algorithm::new_hmac(jsonwebtokens::AlgorithmID::HS256, "secret").unwrap();
        let verifier = Verifier::create()
            .issuer("https://www.mrfy.io")
            .audience(format!("https://{}", host))
            .build()?;
        let claims: Value = verifier.verify(&token, &alg)?;
        Ok(Self {
            id: claims
                .get("sub")
                .ok_or(Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    "Invalid Data",
                )))?
                .as_str()
                .ok_or(Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    "Invalid Data",
                )))?
                .parse()?,
            email: claims
                .get("email")
                .ok_or(Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    "Invalid Data",
                )))?
                .as_str()
                .ok_or(Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    "Invalid Data",
                )))?
                .to_owned(),
            name: claims
                .get("name")
                .and_then(|x| x.as_str().and_then(|x| Some(x.to_owned()))),
            groups: None,
        })
    }
}

pub struct UserDb {
    /// Connection to a databse
    /// This database should have a users table defined
    /// DEPRECATED --- CREATE TABLE users (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, username TEXT NOT NULL, password TEXT NOT NULL, name TEXT NOT NULL);
    ///
    /// CREATE TABLE users (id INTEGER NOT NULL PRIMARY KEY, email TEXT UNIQUE NOT NULL, salt TEXT NOT NULL, password TEXT NOT NULL, obj JSON NOT NULL);
    /// CREATE TABLE clients (client_id TEXT NOT NULL, obj BLOB NOT NULL);
    connection: Arc<Mutex<sqlite::Connection>>,
}

impl UserDb {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let connection = sqlite::open("users.db")?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub async fn list_users(&self) -> Result<Vec<User>, Box<dyn Error>> {
        let connection = self.connection.lock().await;
        static QUERY: &str = "SELECT id, email, json_extract(obj, '$.name') name FROM users;";
        let mut statement = connection.prepare(QUERY)?;

        let mut users: Vec<User> = Vec::new();
        while let Ok(sqlite::State::Row) = statement.next() {
            users.push(User::new_from_row(&mut statement)?);
        }

        Ok(users)
    }

    /// Creates a new user from a given User
    /// Not all properties will be used
    /// Returns the resulting User
    pub async fn create_user(&self, user: User) -> Result<User, Box<dyn Error>> {
        todo!()
    }

    pub async fn authenticate_user(
        &self,
        email: Option<&String>,
        password: Option<&String>,
    ) -> Result<User, Box<dyn Error>> {
        let connection = self.connection.lock().await;
        static QUERY: &str = "SELECT id, email, json_extract(obj, '$.name') name FROM users WHERE email = ? AND password = ?;";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((
            1,
            email
                .ok_or(Box::new(IoError::new(ErrorKind::NotFound, "Not Found")))?
                .as_str(),
        ))?;
        statement.bind((
            2,
            password
                .ok_or(Box::new(IoError::new(ErrorKind::NotFound, "Not Found")))?
                .as_str(),
        ))?;

        let Ok(sqlite::State::Row) = statement.next() else {
            return Err(Box::new(IoError::new(ErrorKind::NotFound, "Not Found")));
        };

        User::new_from_row(&mut statement)
    }

    /// Reads a user from the database from their id
    pub async fn read_user(&self, id: i64) -> Result<User, Box<dyn Error>> {
        let connection = self.connection.lock().await;
        static QUERY: &str =
            "SELECT id,email, json_extract(obj, '$.name') name FROM users WHERE id = ?;";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((1, id))?;

        let Ok(sqlite::State::Row) = statement.next() else {
            return Err(Box::new(IoError::new(ErrorKind::NotFound, "Not Found")));
        };

        User::new_from_row(&mut statement)
    }

    pub fn update_user(&self, id: i64, user: User) -> Result<User, Box<dyn Error>> {
        todo!()
    }
    pub fn delete_user(&self, id: i64) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

pub async fn users_list(
    State(state): State<MyServerContext>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let Ok(users) = state.user_db.list_users().await else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(users))
}

pub async fn users_read(
    State(state): State<MyServerContext>,
    Path(id): Path<i64>,
) -> Result<Json<User>, StatusCode> {
    let Ok(user) = state.user_db.read_user(id).await else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(user))
}
