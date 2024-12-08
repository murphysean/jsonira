use std::sync::Arc;
use std::error::Error;

use tokio::sync::Mutex;


pub struct ChatDb {
    /// Connection to a databse
    /// This database should have a users table defined
    /// DEPRECATED --- CREATE TABLE users (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, username TEXT NOT NULL, password TEXT NOT NULL, name TEXT NOT NULL);
    ///
    /// CREATE TABLE rooms (id INTEGER NOT NULL PRIMARY KEY, email TEXT UNIQUE NOT NULL, salt TEXT NOT NULL, password TEXT NOT NULL, obj JSON NOT NULL);
    /// CREATE TABLE messages (client_id TEXT NOT NULL, obj BLOB NOT NULL);
    connection: Arc<Mutex<sqlite::Connection>>,
}

impl ChatDb {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let connection = sqlite::open("chat.db")?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /* 
    pub async fn select_rooms(&self) -> Result<Vec<User>, Box<dyn Error>> {
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
        let connection = self.connection.lock().await;
        static QUERY: &str = "INSERT INTO users (email, salt, password, obj) VALUES (?,'salt','password',?) RETURNING id;";
        let mut statement = connection.prepare(QUERY)?;
        let bstr = to_string(&user)?;
        statement.bind((1, user.email.as_str()))?;
        statement.bind((2, bstr.as_str()))?;
        let Ok(sqlite::State::Row) = statement.next() else {
            return Err(Box::new(IoError::new(
                ErrorKind::InvalidInput,
                "Invalid Input",
            )));
        };
        let user = User {
            id: statement.read("id")?,
            email: user.email,
            name: user.name,
            groups: user.groups,
        };
        Ok(user)
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
    }*/
}