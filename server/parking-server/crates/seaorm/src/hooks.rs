use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction};

use crate::errors::OrmResult;
use crate::types::{CreateFields, UpdateFields};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookControl {
    Continue,
    Cancel,
}

impl HookControl {
    #[inline]
    pub fn is_cancelled(self) -> bool {
        matches!(self, Self::Cancel)
    }
}

pub struct HookCtx<'a> {
    pub db: &'a DatabaseConnection,
    pub tx: Option<&'a DatabaseTransaction>,
}

#[async_trait]
pub trait OrmHook<M: Send + Sync>: Send + Sync {
    async fn before_insert(
        &self,
        create: &mut CreateFields,
        ctx: &HookCtx<'_>,
    ) -> OrmResult<HookControl> {
        let _ = (create, ctx);
        Ok(HookControl::Continue)
    }

    async fn after_insert(&self, model: &M, ctx: &HookCtx<'_>) -> OrmResult<()> {
        let _ = (model, ctx);
        Ok(())
    }

    async fn before_update(
        &self,
        id: &uuid::Uuid,
        update: &mut UpdateFields,
        ctx: &HookCtx<'_>,
    ) -> OrmResult<HookControl> {
        let _ = (id, update, ctx);
        Ok(HookControl::Continue)
    }

    async fn after_update(&self, model: &M, ctx: &HookCtx<'_>) -> OrmResult<()> {
        let _ = (model, ctx);
        Ok(())
    }

    async fn before_delete(&self, model: &M, ctx: &HookCtx<'_>) -> OrmResult<HookControl> {
        let _ = (model, ctx);
        Ok(HookControl::Continue)
    }

    async fn after_delete(&self, model: &M, ctx: &HookCtx<'_>) -> OrmResult<()> {
        let _ = (model, ctx);
        Ok(())
    }
}
