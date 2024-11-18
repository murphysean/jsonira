use chrono::{Duration, Utc};
use warp::http::{response, Response, StatusCode};
use jsonwebtokens::{encode, Algorithm, Verifier};
use serde::Serialize;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use warp::reply::{self, Reply};
use std::convert::Infallible;
use std::io::Error as IoError;
use std::sync::Arc;
use std::{error::Error, io::ErrorKind};
use tokio::sync::Mutex;
use uuid::Uuid;

use sqlite::{State, Statement};

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
    pub username: String,
    password: Option<String>,

    pub name: String,
}

impl User {
    pub fn new() -> Self {
        Self::default()
    }

    fn new_from_row(statement: &mut Statement) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            id: statement.read("id")?,
            username: statement.read("username")?,
            password: None,
            name: statement.read("name")?,
        })
    }

    pub fn create_token(&self) -> String {
        let alg = Algorithm::new_hmac(jsonwebtokens::AlgorithmID::HS256, "secret").unwrap();
        let header = json!({"alg": alg.name()});
        let then = Utc::now() + Duration::hours(8);
        let claims = json!({
            "iss": "https://www.mrfy.io",
            "sub": &self.id,
            "aud": "https://www.mrfy.io",
            "iat": Utc::now(),
            "nbf": Utc::now(),
            "exp": then,
            "jti": Uuid::new_v4(),
            "sid": Uuid::new_v4(),
            "name": &self.name,
            "username": &self.username,
        });
        encode(&header, &claims, &alg).unwrap()
    }

    pub fn new_from_token(token: String) -> Result<User, Box<dyn Error>> {
        let alg = Algorithm::new_hmac(jsonwebtokens::AlgorithmID::HS256, "secret").unwrap();
        let verifier = Verifier::create()
            .issuer("https://www.mrfy.io")
            .audience("www.mrfy.io")
            .build()?;
        let claims: Value = verifier.verify(&token, &alg)?;
        Ok(Self {
            id: claims
                .get("user_id")
                .ok_or(Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    "Invalid Data",
                )))?
                .as_i64()
                .ok_or(Box::new(IoError::new(
                    ErrorKind::InvalidData,
                    "Invalid Data",
                )))?,
            username: claims
                .get("username")
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
            password: None,
            name: claims
                .get("name")
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
        })
    }
}

pub struct UserDb {
    /// Connection to a databse
    /// This database should have a users table defined
    /// CREATE TABLE users (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, username TEXT NOT NULL, password TEXT NOT NULL, name TEXT NOT NULL);
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
        static QUERY: &str = "SELECT id,username,name FROM users";
        let mut statement = connection.prepare(QUERY)?;

        let mut users: Vec<User> = Vec::new();
        while let Ok(State::Row) = statement.next() {
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
        username: Option<&String>,
        password: Option<&String>,
    ) -> Result<User, Box<dyn Error>> {
        let connection = self.connection.lock().await;
        static QUERY: &str =
            "SELECT id,username,name FROM users WHERE username = ? AND password = ?";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((
            1,
            username
                .ok_or(Box::new(IoError::new(ErrorKind::NotFound, "Not Found")))?
                .as_str(),
        ))?;
        statement.bind((
            2,
            password
                .ok_or(Box::new(IoError::new(ErrorKind::NotFound, "Not Found")))?
                .as_str(),
        ))?;

        let Ok(State::Row) = statement.next() else {
            return Err(Box::new(IoError::new(ErrorKind::NotFound, "Not Found")));
        };

        User::new_from_row(&mut statement)
    }

    /// Reads a user from the database from their id
    pub async fn read_user(&self, id: i64) -> Result<User, Box<dyn Error>> {
        let connection = self.connection.lock().await;
        static QUERY: &str = "SELECT id,username,name FROM users WHERE id = ?";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((1, id))?;

        let Ok(State::Row) = statement.next() else {
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

pub async fn users_list(db: Arc<UserDb>) -> Result<impl Reply, Infallible> {
    let Ok(users) = db.list_users().await else {
        return Ok(reply::with_status(
            reply::json(&SimpleErr::new(String::from("Not Found"))),
            StatusCode::NOT_FOUND,
        ));
    };
    Ok(reply::with_status(
        reply::json(&users),
        StatusCode::OK,
    ))
}

pub async fn users_read(db: Arc<UserDb>, id: i64) -> Result<impl Reply, Infallible> {
    let Ok(user) = db.read_user(id).await else{
        return Ok(reply::with_status(
            reply::json(&SimpleErr::new(String::from("Not Found"))),
            StatusCode::NOT_FOUND,
        ));
    }
    Ok(reply::with_status(
        reply::json(&user),
        StatusCode::OK,
    ))
}