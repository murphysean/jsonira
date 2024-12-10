use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Room {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub created: Option<u64>,
    pub users: Option<Vec<u64>>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: Option<u64>,
    pub user_id: Option<u64>,
    pub message: Option<String>,
    pub reactions: Option<Vec<Reaction>>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Reaction {
    pub reaction: Option<String>,
    pub user_ids: Option<Vec<u64>>,
}
