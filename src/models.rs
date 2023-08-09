use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// Username as associated with github (could potenitally decouple from github in future)
    pub username: String,

    // /// Hashed password
    // pub hashed_pass: String,
    /// List of installations the user has permission to manage
    pub github_installations: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
    pub owner: String,
    pub repo: String,
    pub issue_id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bounty {
    /// Compensantion of the reward
    pub reward: u64,
    /// Owner is a github user id
    pub owner: String,
    /// github node_id of the original issue
    pub issue: Issue,
}
