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
pub enum BountyStatus {
    Open,
    Completed,
    Closed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bounty {
    /// The user that owns this bounty
    pub user: String,
    /// Compensantion of the reward
    pub reward: u64,
    /// github node_id of the original issue
    pub issue: Issue,
    /// The current status of the bounty
    pub status: BountyStatus,
    pub title: String,
    pub description: String,
    /// List of tags on the issue
    pub labels: Vec<String>,
    pub created: chrono::DateTime<chrono::offset::Utc>,
    /// Token ID of the bounty NFT the user has created
    pub token_id: u64,
}
