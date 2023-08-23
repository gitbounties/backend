use log::info;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

use crate::models::{Bounty, BountyStatus, Issue, User};

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
}

pub async fn migrate_dummy(db_conn: &DBConnection) {
    let username = "MrPicklePinosaur";
    let installations: Vec<usize> = vec![40304727];

    // TODO lazy way to handle CREATE IF NOT EXIST: just ignore the error
    let _res: User = db_conn
        .create(("Users", username))
        .content(User {
            username: username.to_string(),
            github_installations: installations,
            wallet_address: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
                .parse()
                .unwrap(),
        })
        .await
        .unwrap();

    let _res: Bounty = db_conn
        .create("Bounty")
        .content(Bounty {
            user: "MrPicklePinosaur".into(),
            reward: 1,
            issue: Issue {
                owner: "MrPicklePinosaur".into(),
                repo: "testing".into(),
                issue_id: 1,
            },
            status: BountyStatus::Open,
            title: "My Test Issue".into(),
            description: "description of my issue".into(),
            labels: vec![],
            created: chrono::offset::Utc::now(),
            token_id: 1,
        })
        .await
        .unwrap();
}
