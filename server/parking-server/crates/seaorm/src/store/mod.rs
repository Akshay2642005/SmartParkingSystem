use std::{sync::Arc, time::Duration};

use sea_orm::{
    ConnectOptions, Database, DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait,
};

use crate::{
    errors::{OrmResult, map_db_err},
    store::repository::Repository,
};

pub mod entities;
pub mod repository;

#[derive(Clone)]
pub struct SeaOrmStore {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmStore {
    pub async fn connect(store: &configuration::DatabaseConfig) -> OrmResult<Self> {
        let mut opts = ConnectOptions::new(&store.url);
        opts.max_connections(store.max_connections)
            .min_connections(store.min_connections)
            .connect_timeout(Duration::from_secs(store.connect_timeout_secs))
            .acquire_timeout(Duration::from_secs(store.acquire_timeout_secs))
            .idle_timeout(Duration::from_secs(store.idle_timeout_secs))
            .max_lifetime(Duration::from_secs(store.max_lifetime_secs))
            .sqlx_logging(store.sqlx_logging)
            .sqlx_logging_level(log::LevelFilter::Debug);

        tracing::info!("connecting to postgres");

        let db = Database::connect(opts).await.map_err(map_db_err)?;

        tracing::info!("store ready!");
        Ok(Self { db: Arc::new(db) })
    }

    #[must_use]
    pub fn from_connection(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    #[must_use]
    pub fn into_connection(self) -> Option<DatabaseConnection> {
        Arc::try_unwrap(self.db).ok()
    }

    #[must_use]
    pub fn db(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }

    pub async fn ping(&self) -> Result<(), DbErr> {
        self.db.ping().await
    }

    pub fn repository<E>(&self) -> repository::Repository<E>
    where
        E: sea_orm::EntityTrait,
        E::Model: Send + Sync,
    {
        Repository::from_shared(Arc::clone(&self.db))
    }

    pub async fn transaction<F, R>(&self, work: F) -> OrmResult<R>
    where
        F: for<'tx> FnOnce(
                &'tx DatabaseTransaction,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = OrmResult<R>> + Send + 'tx>,
            > + Send,
        R: Send,
    {
        let tx = self.db.as_ref().begin().await.map_err(map_db_err)?;
        match work(&tx).await {
            Ok(v) => {
                tx.commit().await.map_err(map_db_err)?;
                Ok(v)
            }
            Err(e) => {
                tx.rollback().await.map_err(map_db_err)?;
                Err(e)
            }
        }
    }

    pub async fn close(self) -> OrmResult<()> {
        match Arc::try_unwrap(self.db) {
            Ok(db) => db.close().await.map_err(map_db_err),
            Err(_) => Err(crate::errors::OrmError::Database(
                "database connection is still in use".to_string(),
            )),
        }
    }
}
