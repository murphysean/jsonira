

struct Room{
    pub id: Option<u64>,
    pub name: Option<String>,
    pub created: Option<u64>,
    pub users: Option<Vec<u64>>,
}

struct Message{
    pub id: Option<u64>,
    pub user_id: Option<u64>,
    pub message: Option<String>,
    pub reactions: Option<Vec<Reaction>>,
}

struct Reaction{
    pub reaction: Option<String>,
    pub user_ids: Option<Vec<u64>>,
}