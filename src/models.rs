use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    /// Username as associated with github (could potenitally decouple from github in future)
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct Issue<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub url: &'a str,
    pub node_id: &'a str,
}
