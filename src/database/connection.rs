use sqlx::{postgres::PgPoolOptions, Postgres, Pool};
use dotenv::dotenv;
use std::env;
pub struct DB;

impl DB {
  pub  async fn connection() -> Pool<Postgres> {

        dotenv().ok();

        let env_url_db=  env::var("DATABASE_URL")
        .expect("DATABASE_URL cannot be empty");
        let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env_url_db)
        .await
        .expect("Failed to connect to database");

    pool
    }
}