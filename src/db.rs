use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub type Db = Pool<Postgres>;

pub async fn db_connect(database_url: &str) -> Pool<Postgres> {
    PgPoolOptions::new()
        .connect(database_url)
        .await
        .expect("Unable to connect to database")
}
