use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    /// Username as associated with github (could potenitally decouple from github in future)
    pub username: String,
    // /// Unique identifier for github account
    // pub github_id: String,
}

#[derive(Debug, Serialize)]
pub struct Issue<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub url: &'a str,
    pub node_id: &'a str,
}
