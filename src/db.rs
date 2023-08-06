use log::info;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

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
