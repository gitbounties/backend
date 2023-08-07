use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    /// Username as associated with github (could potenitally decouple from github in future)
    pub username: String,
}

// we probably don't need a struct for issue?
#[derive(Debug, Serialize)]
pub struct Issue<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub url: &'a str,
    pub node_id: &'a str,
}

#[derive(Debug, Serialize)]
pub struct Bounty<'a> {
    /// Compensantion of the reward
    pub reward: u64,
    /// Owner is a github user id
    pub owner: &'a str,
    /// github node_id of the original issue
    pub issue: &'a str,
}
