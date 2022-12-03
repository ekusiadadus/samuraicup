use diesel::{r2d2, sqlite::SqliteConnection};
use lazy_init::LazyTransform;
use std::sync::Arc;

#[derive(Clone)]
pub struct SqliteConnPool(
    Arc<LazyTransform<Config, r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>>>,
);

struct Config {
    database_url: String,
    size_conn_pool: u32,
}

fn initialize(config: Config) -> r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>> {
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new(config.database_url);
    r2d2::Pool::builder()
        .max_size(config.size_conn_pool)
        .build(manager)
        .expect("Failed to create pool.")
}

impl SqliteConnPool {
    pub fn new(database_url: String, size_conn_pool: u32) -> SqliteConnPool {
        SqliteConnPool(Arc::new(LazyTransform::new(Config {
            database_url,
            size_conn_pool,
        })))
    }

    pub fn ensure_initialized(&self) {
        self.get_connection();
    }

    pub fn get_connection(
        &self,
    ) -> r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>> {
        self.0.get_or_create(initialize).get().unwrap()
    }
}
