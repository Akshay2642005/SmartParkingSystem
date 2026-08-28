//! Thin repository helpers for generated SeaORM entities.
//!
//! Repositories are cheap to construct (they hold an `Arc<DatabaseConnection>`
//! clone). Generated entities remain the source of truth for table shape,
//! columns, relations, and active models.
//!
//! # Usage
//!
//! ```rust,ignore
//! let repo = store.repository::<todos::Entity>();
//!
//! // Basic CRUD
//! let created = repo.insert(active_model).await?;
//! let found   = repo.get_by_id(created.id).await?;
//! let exists  = repo.exists(todos::Column::Title.eq("buy milk")).await?;
//! let count   = repo.count(todos::Column::Done.eq(false)).await?;
//!
//! // Paginated list with caller-supplied filter
//! let page = repo.list(&Page::default(), |q| {
//!     q.filter(todos::Column::Done.eq(false))
//! }).await?;
//!
//! // Transaction-aware insert
//! store.transaction(|tx| Box::pin(async move {
//!     repo.insert_in_tx(tx, active_model).await?;
//!     Ok(())
//! })).await?;
//! ```

use std::{marker::PhantomData, sync::Arc};

use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, IntoActiveModel,
    PaginatorTrait, PrimaryKeyTrait, QueryFilter, Select, TransactionTrait,
    sea_query::IntoCondition,
};

use crate::errors::{OrmError, OrmResult, map_db_err};
use crate::types::{Page, Paginated};

/// Generic typed repository for generated SeaORM entity `E`.
pub struct Repository<E: EntityTrait> {
    db: Arc<DatabaseConnection>,
    _entity: PhantomData<E>,
}

impl<E: EntityTrait> Clone for Repository<E> {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            _entity: PhantomData,
        }
    }
}

impl<E> Repository<E>
where
    E: EntityTrait,
    E::Model: Send + Sync,
{
    /// Create a repository from an existing SeaORM database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self::from_shared(Arc::new(db))
    }

    /// Create a repository from a shared SeaORM database connection.
    pub fn from_shared(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db,
            _entity: PhantomData,
        }
    }

    /// Insert an active model and return the inserted model.
    pub async fn insert<A>(&self, active_model: A) -> OrmResult<E::Model>
    where
        A: ActiveModelTrait<Entity = E> + Send,
        E::Model: IntoActiveModel<A>,
    {
        E::insert(active_model)
            .exec_with_returning(self.db.as_ref())
            .await
            .map_err(map_db_err)
    }

    /// Insert inside an open transaction.
    pub async fn insert_in_tx<A>(
        &self,
        tx: &DatabaseTransaction,
        active_model: A,
    ) -> OrmResult<E::Model>
    where
        A: ActiveModelTrait<Entity = E> + Send,
        E::Model: IntoActiveModel<A>,
    {
        E::insert(active_model)
            .exec_with_returning(tx)
            .await
            .map_err(map_db_err)
    }

    /// Update an active model and return the updated model.
    ///
    /// The caller must populate the primary key on the active model.
    pub async fn update<A>(&self, active_model: A) -> OrmResult<E::Model>
    where
        A: ActiveModelTrait<Entity = E> + Send,
        E::Model: IntoActiveModel<A>,
    {
        E::update(active_model)
            .exec(self.db.as_ref())
            .await
            .map_err(map_db_err)
    }

    /// Update inside an open transaction.
    pub async fn update_in_tx<A>(
        &self,
        tx: &DatabaseTransaction,
        active_model: A,
    ) -> OrmResult<E::Model>
    where
        A: ActiveModelTrait<Entity = E> + Send,
        E::Model: IntoActiveModel<A>,
    {
        E::update(active_model).exec(tx).await.map_err(map_db_err)
    }

    /// Insert or update: tries to insert; on unique-key conflict, applies the
    /// caller-supplied `on_conflict` strategy.
    ///
    /// ```rust,ignore
    /// use sea_orm::sea_query::{OnConflict, Expr};
    ///
    /// repo.upsert(active, |stmt| {
    ///     stmt.value(todos::Column::Title, Expr::val("new title"))
    /// }).await?;
    /// ```
    pub async fn upsert<A, F>(&self, active_model: A, on_conflict: F) -> OrmResult<E::Model>
    where
        A: ActiveModelTrait<Entity = E> + Send,
        E::Model: IntoActiveModel<A>,
        F: FnOnce(sea_orm::sea_query::OnConflict) -> sea_orm::sea_query::OnConflict + Send,
    {
        let conflict = on_conflict(sea_orm::sea_query::OnConflict::new());
        E::insert(active_model)
            .on_conflict(conflict)
            .exec_with_returning(self.db.as_ref())
            .await
            .map_err(map_db_err)
    }

    /// Insert multiple rows in a single statement and return the inserted models.
    ///
    /// Returns an empty `Vec` when `models` is empty (no round-trip).
    pub async fn bulk_insert<A>(&self, models: Vec<A>) -> OrmResult<Vec<E::Model>>
    where
        A: ActiveModelTrait<Entity = E> + Send,
        E::Model: IntoActiveModel<A>,
    {
        if models.is_empty() {
            return Ok(vec![]);
        }
        #[allow(deprecated)]
        E::insert_many(models)
            .exec_with_returning_many(self.db.as_ref())
            .await
            .map_err(map_db_err)
    }

    /// Bulk insert inside an open transaction.
    pub async fn bulk_insert_in_tx<A>(
        &self,
        tx: &DatabaseTransaction,
        models: Vec<A>,
    ) -> OrmResult<Vec<E::Model>>
    where
        A: ActiveModelTrait<Entity = E> + Send,
        E::Model: IntoActiveModel<A>,
    {
        if models.is_empty() {
            return Ok(vec![]);
        }
        #[allow(deprecated)]
        E::insert_many(models)
            .exec_with_returning_many(tx)
            .await
            .map_err(map_db_err)
    }

    /// Return all rows (no filter, no pagination). Use with care on large tables.
    pub async fn get_all(&self) -> OrmResult<Vec<E::Model>> {
        E::find().all(self.db.as_ref()).await.map_err(map_db_err)
    }

    /// Find a model by primary key; returns `None` if not found.
    pub async fn find_by_id<T>(&self, id: T) -> OrmResult<Option<E::Model>>
    where
        T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        E::find_by_id(id)
            .one(self.db.as_ref())
            .await
            .map_err(map_db_err)
    }

    /// Find a model by primary key; returns `OrmError::NotFound` if missing.
    pub async fn get_by_id<T>(&self, id: T) -> OrmResult<E::Model>
    where
        T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        self.find_by_id(id).await?.ok_or(OrmError::NotFound)
    }

    /// Return the first row matching an arbitrary filter, or `None`.
    ///
    /// ```rust,ignore
    /// let todo = repo.find_one(todos::Column::Title.eq("buy milk")).await?;
    /// ```
    pub async fn find_one<C>(&self, condition: C) -> OrmResult<Option<E::Model>>
    where
        C: IntoCondition + Send,
    {
        E::find()
            .filter(condition)
            .one(self.db.as_ref())
            .await
            .map_err(map_db_err)
    }

    /// Return the first row matching an arbitrary filter; `OrmError::NotFound` if missing.
    pub async fn get_one<C>(&self, condition: C) -> OrmResult<E::Model>
    where
        C: IntoCondition + Send,
    {
        self.find_one(condition).await?.ok_or(OrmError::NotFound)
    }

    /// Paginated list. `filter_fn` lets the caller inject filters, ordering,
    /// joins, or selects without exposing raw SQL.
    pub async fn list<F>(&self, page: &Page, filter_fn: F) -> OrmResult<Paginated<E::Model>>
    where
        F: FnOnce(Select<E>) -> Select<E> + Send,
    {
        let q = filter_fn(E::find());
        let pag = q.paginate(self.db.as_ref(), page.per_page());
        let total = pag.num_items().await.map_err(map_db_err)?;
        let items = pag
            .fetch_page(page.zero_indexed())
            .await
            .map_err(map_db_err)?;
        Ok(Paginated::new(items, total, page))
    }

    /// Return the total number of rows matching `condition`.
    ///
    /// ```rust,ignore
    /// let pending = repo.count(todos::Column::Done.eq(false)).await?;
    /// ```
    pub async fn count<C>(&self, condition: C) -> OrmResult<u64>
    where
        C: IntoCondition + Send,
    {
        E::find()
            .filter(condition)
            .count(self.db.as_ref())
            .await
            .map_err(map_db_err)
    }

    /// Return `true` if at least one row matches `condition`.
    pub async fn exists<C>(&self, condition: C) -> OrmResult<bool>
    where
        C: IntoCondition + Send,
    {
        self.count(condition).await.map(|n| n > 0)
    }

    /// Delete by primary key; returns the number of affected rows.
    pub async fn delete_by_id<T>(&self, id: T) -> OrmResult<u64>
    where
        T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        let result = E::delete_by_id(id)
            .exec(self.db.as_ref())
            .await
            .map_err(map_db_err)?;
        Ok(result.rows_affected)
    }

    /// Delete by primary key inside an open transaction.
    pub async fn delete_by_id_in_tx<T>(&self, tx: &DatabaseTransaction, id: T) -> OrmResult<u64>
    where
        T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        let result = E::delete_by_id(id).exec(tx).await.map_err(map_db_err)?;
        Ok(result.rows_affected)
    }

    /// Delete all rows matching `condition`; returns the number of affected rows.
    pub async fn delete_many<C>(&self, condition: C) -> OrmResult<u64>
    where
        C: IntoCondition + Send,
    {
        let result = E::delete_many()
            .filter(condition)
            .exec(self.db.as_ref())
            .await
            .map_err(map_db_err)?;
        Ok(result.rows_affected)
    }

    /// Execute `work` inside a single ACID transaction managed by this repository.
    ///
    /// Commits on `Ok(_)`, rolls back on `Err(_)`.
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

    /// Borrow the underlying database connection for custom SeaORM queries.
    pub fn db(&self) -> &DatabaseConnection {
        self.db.as_ref()
    }
}

/// Convenience constructor — prefer `store.repository::<E>()` at call sites.
pub fn repo<R>(db: DatabaseConnection) -> Repository<R>
where
    R: EntityTrait,
    R::Model: Send + Sync,
{
    Repository::new(db)
}
