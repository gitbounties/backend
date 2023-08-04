use log::info;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

pub async fn connect(connection_string: &str) -> surrealdb::Result<()> {
    let db = Surreal::new::<Ws>(connection_string).await?;

    db.signin(Root {
        username: "admin",
        password: "password",
    })
    .await?;

    db.use_ns("test").use_db("test").await?;

    info!("Successfully connected to database");

    Ok(())
}
