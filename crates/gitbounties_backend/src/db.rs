use log::info;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

use crate::models::User;

pub type DBConnection = Surreal<Client>;

pub async fn connect(
    connection_string: &str,
    username: &str,
    password: &str,
    namespace: &str,
    database: &str,
) -> surrealdb::Result<DBConnection> {
    let db = Surreal::new::<Ws>(connection_string).await?;

    db.signin(Root { username, password }).await?;

    db.use_ns(namespace).use_db(database).await?;

    info!("Successfully connected to database");

    Ok(db)
}

pub async fn user_register() {}

/// Initialize database
pub async fn migrate(db_conn: &DBConnection) {
    // initalize with some dummy data

    /*
    let username = "MrPicklePinosaur";
    let installations: Vec<usize> = vec![40304727];

    // TODO lazy way to handle CREATE IF NOT EXIST: just ignore the error
    let res: Result<User, surrealdb::Error> = db_conn
        .create(("Users", username))
        .content(User {
            username: username.to_string(),
            github_installations: installations,
            wallet_address:
        })
        .await;
    */
}
