use std::{fmt::Debug, str::FromStr, sync::Arc};

use color_eyre::Result;
use eyre::{eyre, Context};
use jiff::{Span, Timestamp};
use serde_json::{from_str, from_value, to_string, Value};
use sqlite::Statement;
use tokio::sync::Mutex;

use crate::model::{
    subject::Subject,
    task::{Task, TaskPriority, TaskState},
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
    pub async fn read_tasks(
        &self,
        limit: i64,
        offset: i64,
        circles: Option<Vec<String>>,
        tags: Option<Vec<String>>,
    ) -> Result<Vec<Task>> {
        let Some(circles) = circles else {
            return Ok(vec![]);
        };
        if circles.is_empty() {
            return Ok(vec![]);
        }
        let Some(tags) = tags else {
            return Ok(vec![]);
        };
        if tags.is_empty() {
            return Ok(vec![]);
        }
        let circles_string = circles
            .iter()
            .enumerate()
            .map(|(i, _)| format!(":circle{}", i).to_string())
            .collect::<Vec<_>>()
            .join(",");
        let tags_string = tags
            .iter()
            .enumerate()
            .map(|(i, _)| format!(":tag{}", i).to_string())
            .collect::<Vec<_>>()
            .join(",");
        let connection = self.connection.lock().await;
        let query = format!(
            r#"
        SELECT
            tasks.id,
            json_extract(tasks.obj, '$.title') title,
            json_extract(tasks.obj, '$.circle') circle,
            json_extract(tasks.obj, '$.reporter') reporter,
            json_extract(tasks.obj, '$.assignee') assignee,
            json_extract(tasks.obj, '$.priority') priority,
            json_extract(tasks.obj, '$.estimate') estimate,
            json_extract(tasks.obj, '$.points') points,
            json_extract(tasks.obj, '$.state') state,
            json_extract(tasks.obj, '$.tags') tags,
            json_extract(tasks.obj, '$.created') created,
            json_extract(tasks.obj, '$.updated') updated,
            json_extract(tasks.obj, '$.due') due
        FROM
            tasks,
            json_each(json_extract(tasks.obj, '$.tags')) tags
        WHERE
            circle IN ({})
        AND
            tags.value IN ({})
        ORDER BY
            created
        LIMIT :limit OFFSET :offset;"#,
            circles_string, tags_string
        );

        let mut statement = connection.prepare(query)?;
        for (i, s) in circles.iter().enumerate() {
            statement.bind((format!(":circle{}", i).as_str(), s.as_str()))?;
        }
        for (i, s) in tags.iter().enumerate() {
            statement.bind((format!(":tag{}", i).as_str(), s.as_str()))?;
        }
        statement.bind((":limit", limit))?;
        statement.bind((":offset", offset))?;

        let mut tasks: Vec<Task> = Vec::new();
        while let Ok(sqlite::State::Row) = statement.next() {
            let mut task = Task::default();
            task.id = statement.read("id")?;
            task.title = statement.read("title")?;
            task.circle = statement.read("circle")?;
            task.reporter = Self::read_subject(&statement, "reporter")?;
            task.assignee = Self::read_subject(&statement, "assignee")?;
            task.priority = Self::read_json_value(&statement, "priority")
                .wrap_err("unexpected value in priority column")
                .and_then(|ov| match ov {
                    Some(v) => Ok(Some(from_value(v)?)),
                    None => Ok(None),
                })?;
            task.estimate = Self::read_json_value(&statement, "estimate")
                .wrap_err("unexpected value in estimate column")
                .and_then(|ov| match ov {
                    Some(v) => Ok(Some(from_value(v)?)),
                    None => Ok(None),
                })?;
            task.points = statement.read("points")?;
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
            task.tags = Self::read_json_value(&statement,"tags")
                .wrap_err("unexpected value in tags column")
                .and_then(|ov| match ov{
                    Some(v) => {
                        let ret: Vec<String> = from_value(v)?;
                        return Ok(Some(ret));
                    },
                    None => Ok(None),
                })?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    //I need a function that will read a valid json value from the column
    //or convert string literal -> json string
    //fn read_json_value
    fn read_json_value(statement: &Statement, index: &str) -> Result<Option<Value>> {
        let result = statement.read::<Option<String>, &str>(index);
        match result {
            Ok(os) => {
                match os {
                    Some(s) => {
                        //TODO Try to parse as json
                        let result = from_str::<Value>(&s);
                        match result {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Ok(Some(Value::String(s))),
                        }
                    }
                    None => Ok(None),
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn read_subject(statement: &Statement, index: &str) -> Result<Option<Subject>> {
        //Eary return if it is just an integer user id, this eagerly returns a 0 for non-integers
        if let Some(val) = statement.read::<i64, &str>(index).ok().and_then(|v| {
            if v == 0 {
                return None;
            } else {
                return Some(Subject::UserId(v));
            }
        }) {
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
