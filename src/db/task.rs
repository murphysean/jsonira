use std::{fmt::Debug, str::FromStr, sync::Arc};

use color_eyre::Result;
use eyre::{eyre, Context};
use jiff::Timestamp;
use serde_json::{from_str, to_string};
use sqlite::Statement;
use tokio::sync::Mutex;

use crate::model::{
    subject::Subject,
    task::{Task, TaskState},
};

#[derive(Clone)]
pub struct TaskDb {
    /// Connection to a databse
    /// This database should have a users table defined
    ///
    /// CREATE TABLE tasks (id INTEGER NOT NULL PRIMARY KEY, obj JSON NOT NULL);
    connection: Arc<Mutex<sqlite::Connection>>,
}

impl Debug for TaskDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskDb").finish()
    }
}

/// Offers CRUD operations against the user, client, and auth store
impl TaskDb {
    pub fn new() -> Result<Self> {
        let connection = sqlite::open("tasks.db")?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    /// Will get an array of tasks
    /// Only returns the id, title, assignee, state, created, updated, due,
    #[tracing::instrument(level = "info")]
    pub async fn read_tasks(&self) -> Result<Vec<Task>> {
        let connection = self.connection.lock().await;
        static QUERY: &str = r#"
        SELECT
            id,
            json_extract(obj, '$.title') title,
            json_extract(obj, '$.assignee') assignee,
            json_extract(obj, '$.reporter') reporter,
            json_extract(obj, '$.state') state,
            json_extract(obj, '$.created') created,
            json_extract(obj, '$.updated') updated,
            json_extract(obj, '$.due') due,
            json_extract(obj, '$.circle') circle
        FROM
            tasks
        WHERE
            circle = :circle
        ORDER BY
            created
        LIMIT :limit OFFSET :offset;"#;

        let mut statement = connection.prepare(QUERY)?;
        statement.bind((":circle", "mrfy-family"))?;
        statement.bind((":limit", 100))?;
        statement.bind((":offset", 0))?;

        let mut tasks: Vec<Task> = Vec::new();
        while let Ok(sqlite::State::Row) = statement.next() {
            let mut task = Task::default();
            task.id = statement.read("id")?;
            task.title = statement.read("title")?;
            task.assignee = Self::read_subject(&statement, "assignee")?;
            task.reporter = Self::read_subject(&statement, "reporter")?;
            task.state = statement
                .read::<Option<String>, &str>("state")
                .wrap_err("unexpected value in state column")
                .and_then(|os| match os {
                    Some(s) => Ok(Some(from_str::<TaskState>(s.as_str())?)),
                    None => Ok(None),
                })?;
            task.created = statement
                .read::<Option<String>, &str>("created")
                .wrap_err("unexpected value in created column")
                .and_then(|os| match os {
                    Some(s) => Ok(Some(Timestamp::from_str(s.as_str())?)),
                    None => Ok(None),
                })?;
            task.updated = statement
                .read::<Option<String>, &str>("updated")
                .wrap_err("unexpected value in updated column")
                .and_then(|os| match os {
                    Some(s) => Ok(Some(Timestamp::from_str(s.as_str())?)),
                    None => Ok(None),
                })?;
            task.due = statement
                .read::<Option<String>, &str>("due")
                .wrap_err("unexpected value in due column")
                .and_then(|os| match os {
                    Some(s) => Ok(Some(s.as_str().parse()?)),
                    None => Ok(None),
                })?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    fn read_subject(statement: &Statement, index: &str) -> Result<Option<Subject>> {
        //Eary return if it is just an integer user id
        if let Some(val) = statement
            .read::<i64, &str>(index)
            .ok()
            .and_then(|v| Some(Subject::UserId(v)))
        {
            return Ok(Some(val));
        }
        //Now see if it's a email subject or a user obj
        let result = statement.read::<Option<String>, &str>(index)?;
        if let Some(s) = result {
            let subject: Subject = from_str(s.as_str())?;
            return Ok(Some(subject));
        } else {
            //If it's a none, then the column is empty, return none
            return Ok(None);
        }
    }

    fn read_timestamp(statement: &Statement, index: &str) -> Option<Timestamp> {
        None
    }

    /// Creates a new task from a given task
    /// Returns the resulting Task
    #[tracing::instrument(level = "info")]
    pub async fn create_task(&self, mut task: Task) -> Result<Task> {
        task.id = None;
        let bstr = to_string(&task)?;
        let connection = self.connection.lock().await;
        static QUERY: &str = "INSERT INTO tasks (obj) VALUES (:obj) RETURNING id;";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((":obj", bstr.as_str()))?;
        let Ok(sqlite::State::Row) = statement.next() else {
            return Err(eyre!("Did not get the inserted id back from create task"));
        };
        task.id = statement.read("id")?;
        Ok(task)
    }

    /// Reads a task from the database given it's id
    #[tracing::instrument(level = "info")]
    pub async fn read_task(&self, id: i64) -> Result<Task> {
        let connection = self.connection.lock().await;
        static QUERY: &str = "SELECT id, obj FROM tasks WHERE id = :id;";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((":id", id))?;

        let Ok(sqlite::State::Row) = statement.next() else {
            return Err(sqlite::Error {
                code: Some(404),
                message: Some(String::from("Empty Result Set")),
            })?;
        };

        let mut task: Task = from_str(statement.read::<String, &str>("obj")?.as_str())?;
        task.id = statement.read("id")?;

        Ok(task)
    }

    #[tracing::instrument(level = "info")]
    pub async fn update_task(&self, id: i64, mut task: Task) -> Result<Task> {
        //Set to none for db storage
        task.id = None;
        let obj = to_string(&task)?;
        let connection = self.connection.lock().await;
        static QUERY: &str = "UPDATE tasks SET obj = :obj WHERE id = :id;";
        let mut statement = connection.prepare(QUERY)?;
        statement.bind((":obj", obj.as_str()))?;
        statement.bind((":id", id))?;

        statement.next()?;
        task.id = Some(id);
        Ok(task)
    }
}
