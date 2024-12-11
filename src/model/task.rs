use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use super::user::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskState {
    Blocked,
    Todo,
    InProgress,
    InReview,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Neutral,
    Trivial,
    Minor,
    Major,
    Critical,
    Blocker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    ///Assigned by the storage layer
    pub id: Option<i64>,
    pub title: String,
    pub description: String,

    pub reporter: Option<User>,
    pub watchers: Option<Vec<User>>,
    //At creation a group can be assigned
    //It can be one of the creating users groups
    //Or it can be 'public' which offers this task to everyone (Everyone is a member of public)
    //If it is none, then only the reporter, assignee, and watchers will have access to this task
    pub circle: Option<String>,

    pub assignee: Option<User>,
    pub priority: Option<TaskPriority>,
    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub estimate: Option<Duration>,
    pub points: Option<i64>,
    pub state: TaskState,

    pub tags: Vec<String>,

    pub created: u64,
    pub updated: u64,
    pub due: Option<u64>,

    pub comments: Vec<Comment>,
    pub reactions: Vec<Reaction>,
    pub reviews: Vec<Review>,

    pub history: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub user: User,
    pub comment: String,
    pub created: i64,
    pub updated: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub user: User,
    pub reaction: String,
    pub created: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewState {
    Reviewing,
    Approved,
    RequestChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub user: User,
    pub review: ReviewState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Grade {
    A,
    B,
    C,
    D,
    F,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rating {
    Grade(Grade),
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rate {
    pub user: User,
    pub rate: Rating,
}

///Actions are things people can do, extensions of sort
/// For example, you could add a verification of the work
/// You could review the task
/// You could grade the job
/// You could submit an estimate or bid
/// Also this should capture edits
/// Ultimately I think I will have this be a set of json-patch documents with a Subject that did it

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub subject: User,
    pub patch: json_patch::Patch,
}
