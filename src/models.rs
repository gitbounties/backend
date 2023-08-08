use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    /// Username as associated with github (could potenitally decouple from github in future)
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bounty {
    /// Compensantion of the reward
    pub reward: u64,
    /// Owner is a github user id
    pub owner: String,
    /// github node_id of the original issue
    pub issue: usize,
}
