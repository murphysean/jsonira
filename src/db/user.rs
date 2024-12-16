use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde_json::{from_str, to_string};
use sqlite::Statement;
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::model::user::User;

#[derive(Error, Debug)]
pub enum UserDbError {
    #[error("invalid input")]
    InvalidInput,
    #[error("no user found")]
    NoneFound,
    #[error("db error")]
    DBError(#[from] sqlite::Error),
    #[error("serde error")]
    SerdeError(#[from] serde_json::Error),
    #[error("hashing error")]
    HashingError(#[from] argon2::password_hash::errors::Error),
}

impl User {
    fn new_from_row(statement: &mut Statement) -> Result<Self, UserDbError> {
        let mut user: User = from_str(statement.read::<String, &str>("obj")?.as_str())?;
        user.id = statement.read("id")?;
        user.email = statement.read("email")?;
        Ok(user)
    }
}

#[derive(Clone)]
pub struct UserDb {
    /// Connection to a databse
    /// This database should have a users table defined
    /// DEPRECATED --- CREATE TABLE users (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, username TEXT NOT NULL, password TEXT NOT NULL, name TEXT NOT NULL);
    ///
    /// CREATE TABLE users (id INTEGER NOT NULL PRIMARY KEY, email TEXT UNIQUE NOT NULL, password TEXT NOT NULL, obj JSON NOT NULL);
    /// CREATE TABLE clients (client_id TEXT NOT NULL, obj BLOB NOT NULL);
    connection: Arc<Mutex<sqlite::Connection>>,
}

impl Debug for UserDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserDb").finish()
    }
}

/// Offers CRUD operations against the user, client, and auth store
impl UserDb {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let connection = sqlite::open("users.db")?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    #[tracing::instrument(level = "info")]
    pub async fn read_users(&self) -> Result<Vec<User>, UserDbError> {
        let connection = self.connection.lock().await;
        static QUERY: &str = "SELECT id, email, obj FROM users;";
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
    #[tracing::instrument(level = "info")]
    pub async fn create_user(
        &self,
        email: String,
        password: String,
        user: User,
    ) -> Result<User, UserDbError> {
        let mut hash: String = String::default();
        {
            let salt = SaltString::generate(OsRng);
            use argon2::PasswordHasher;

            let pwh = Argon2::default().hash_password(password.as_bytes(), &salt)?;
            let pwhs = pwh.serialize();
            hash = pwhs.as_str().to_owned();
        }
        let connection = self.connection.lock().await;
        static QUERY: &str =
            "INSERT INTO users (email, password, obj) VALUES (:email,:password,:obj) RETURNING id;";
        let mut statement = connection.prepare(QUERY)?;
        let bstr = to_string(&user)?;
        statement.bind((":email", user.email.clone().unwrap_or_default().as_str()))?;
        statement.bind((":password", hash.as_str()))?;
        statement.bind((":obj", bstr.as_str()))?;
        let Ok(sqlite::State::Row) = statement.next() else {
            return Err(UserDbError::NoneFound);
        };
        let user = User {
            id: statement.read("id")?,
            email: user.email,
            name: user.name,
            groups: user.groups,
            email_verified: Some(false),
            phone_number_verified: Some(false),
            roles: None,
            entitlements: None,
            circles: None,
            profile: user.profile,
            picture: user.picture,
            website: user.website,
            gender: user.gender,
            birthdate: user.birthdate,
            phone_number: user.phone_number,
        };
        Ok(user)
    }

    /// Reads a user given an email and a password
    #[tracing::instrument(level = "info")]
    pub async fn authenticate_user(
        &self,
        email: &str,
        password: &str,
    ) -> Result<User, UserDbError> {
        let hash = self.get_password(email).await?;

        {
            let pw_hash = PasswordHash::new(hash.as_str())?;
            let algs: &[&dyn PasswordVerifier] = &[&Argon2::default()];
            pw_hash.verify_password(algs, password)?;
        }

        let connection = self.connection.lock().await;
        static QUERY: &str = "SELECT id, email, obj FROM users WHERE email = :email;";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((":email", email))?;

        let Ok(sqlite::State::Row) = statement.next() else {
            return Err(UserDbError::NoneFound);
        };

        User::new_from_row(&mut statement)
    }

    async fn get_password(&self, email: &str) -> Result<String, UserDbError> {
        let connection = self.connection.lock().await;
        static QUERY: &str = "SELECT password FROM users WHERE email = :email;";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((":email", email))?;

        let Ok(sqlite::State::Row) = statement.next() else {
            return Err(UserDbError::NoneFound);
        };

        Ok(statement.read("password")?)
    }

    /// Reads a user from the database from their id
    #[tracing::instrument(level = "info")]
    pub async fn read_user(&self, id: i64) -> Result<User, UserDbError> {
        let connection = self.connection.lock().await;
        static QUERY: &str = "SELECT id, email, obj FROM users WHERE id = ?;";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((1, id))?;

        let Ok(sqlite::State::Row) = statement.next() else {
            return Err(UserDbError::NoneFound);
        };

        User::new_from_row(&mut statement)
    }

    #[tracing::instrument(level = "info")]
    pub async fn update_user(&self, id: i64, mut user: User) -> Result<User, UserDbError> {
        //Set to none for db storage
        user.id = None;
        let obj = to_string(&user)?;
        let connection = self.connection.lock().await;
        static QUERY: &str = "UPDATE users SET obj = ? WHERE id = ?;";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((":obj", obj.as_str()))?;
        statement.bind((":id", id))?;

        statement.next()?;
        user.id = Some(id);
        Ok(user)
    }

    #[tracing::instrument(level = "info")]
    pub async fn delete_user(&self, id: i64) -> Result<(), UserDbError> {
        let connection = self.connection.lock().await;
        static QUERY: &str = "DELETE FROM users WHERE id = ?;";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((1, id))?;
        let Ok(sqlite::State::Row) = statement.next() else {
            return Err(UserDbError::NoneFound);
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHash, PasswordVerifier,
    };

    #[test]
    fn simple_argon2() {
        let password = String::from("9-cheetahs-hunting");
        let salt = SaltString::generate(OsRng);
        //let salt = String::from("saltsaltsalt");
        use argon2::PasswordHasher;

        let pwh = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap();
        println!("Result: {}", &pwh);
        let pwhs = pwh.serialize();
        let hash: String = pwhs.as_str().to_owned();

        let pw_hash = PasswordHash::new(hash.as_str()).unwrap();
        let algs: &[&dyn PasswordVerifier] = &[&Argon2::default()];
        pw_hash.verify_password(algs, password).unwrap();
    }

    #[test]
    fn test_against_password() {
        let password = String::from("9-cheetahs-hunting");
        let hash = String::from("$argon2id$v=19$m=19456,t=2,p=1$izyzjiCywrLhHs5y3sJFXA$bZyhubk51HLYMdDKWsfNptu+cOUXezc7p9w4vRvrXyM");

        let pw_hash = PasswordHash::new(hash.as_str()).unwrap();
        let algs: &[&dyn PasswordVerifier] = &[&Argon2::default()];
        assert!(pw_hash.verify_password(algs, password).is_ok());
    }
}
