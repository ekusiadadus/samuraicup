use crate::error::*;
use crate::infra::SqliteConnPool;

#[derive(Debug)]
pub enum DBExecutorError {
    DBError,
}

impl IServiceError for DBExecutorError {
    fn error_type(&self) -> String {
        use DBExecutorError::*;

        match self {
            DBError => "db_error",
        }
        .to_string()
    }

    fn status_code(&self) -> http::StatusCode {
        use DBExecutorError::*;

        match self {
            DBError => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<diesel::result::Error> for ServiceError {
    fn from(err: diesel::result::Error) -> ServiceError {
        use diesel::result::Error::*;

        match err {
            NotFound => ServiceError::new(RepositoryError::RecordNotFound, err),
            _ => ServiceError::new(DBExecutorError::DBError, err),
        }
    }
}

#[derive(Clone)]
pub struct DBExecutor(SqliteConnPool);
impl DBExecutor {
    pub fn new(database_url: String, size_conn_pool: u32) -> DBExecutor {
        DBExecutor(SqliteConnPool::new(database_url, size_conn_pool))
    }

    pub fn get_connection(
        &self,
    ) -> r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>> {
        self.0.get_connection()
    }
}

#[derive(Clone)]
pub struct DBConnector(DBExecutor);

impl DBConnector {
    pub fn new(executor: DBExecutor) -> DBConnector {
        DBConnector(executor)
    }
    pub async fn ensure_initialized(&self) -> Result<()> {
        let executor = self.0.clone();

        tokio::task::spawn_blocking(move || {
            executor.0.ensure_initialized();
            Ok(())
        })
        .await?
    }

    pub async fn execute<Q: Send + 'static>(&self, query: Q) -> Result<usize>
    where
        Q: diesel::RunQueryDsl<diesel::SqliteConnection>,
        Q: diesel::query_builder::QueryFragment<diesel::sqlite::Sqlite>,
        Q: diesel::query_builder::QueryId,
    {
        let mut conn = self.0.get_connection();
        tokio::task::spawn_blocking(move || {
            let result = query.execute(&mut conn)?;
            Ok(result)
        })
        .await?
    }

    pub async fn first<T: 'static + Send, Q: 'static + Send>(&self, query: Q) -> Result<T>
    where
        Q: diesel::query_dsl::limit_dsl::LimitDsl,
        Q: diesel::RunQueryDsl<diesel::SqliteConnection>,
        diesel::helper_types::Limit<Q>:
            for<'a> diesel::query_dsl::LoadQuery<'a, diesel::SqliteConnection, T>,
    {
        let mut conn = self.0.get_connection();

        tokio::task::spawn_blocking(move || {
            let result = query.first(&mut conn)?;
            Ok(result)
        })
        .await?
    }

    pub async fn load<T: 'static + Send, Q: 'static + Send>(&self, query: Q) -> Result<Vec<T>>
    where
        Q: diesel::RunQueryDsl<diesel::SqliteConnection>,
        Q: for<'a> diesel::query_dsl::LoadQuery<'a, diesel::SqliteConnection, T>,
    {
        let mut conn = self.0.get_connection();

        tokio::task::spawn_blocking(move || {
            let result = query.load(&mut conn)?;
            Ok(result)
        })
        .await?
    }

    // pub async fn load_sql<
    //     T: 'static + Send + diesel::deserialize::QueryableByName<diesel::sqlite::Sqlite>,
    // >(
    //     &self,
    //     query: impl Into<String>,
    // ) -> Result<Vec<T>> {
    //     let mut conn = self.0.get_connection();
    //     let q = query.into();

    //     tokio::task::spawn_blocking(move || {
    //         use diesel::prelude::*;

    //         let result = diesel::sql_query(q).load::<T>(&mut conn)?;
    //         Ok(result)
    //     })
    //     .await?
    // }

    // pub async fn get_result<T: 'static + Send, Q: 'static + Send>(&self, query: Q) -> Result<T>
    // where
    //     Q: diesel::RunQueryDsl<diesel::SqliteConnection>,
    //     Q: for<'a> diesel::query_dsl::LoadQuery<'a, diesel::SqliteConnection, T>,
    // {
    //     let mut conn = self.0.get_connection();

    //     tokio::task::spawn_blocking(move || {
    //         let result = query.get_result(&mut conn)?;
    //         Ok(result)
    //     })
    //     .await?
    // }
}
