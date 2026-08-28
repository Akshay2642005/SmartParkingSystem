//! Latest accepted snapshot per `(site, section)`.
//!
//! `last_seq` and `slot_count` are the durable form of the ingest guards from
//! `architecture/communication.md` § Backend Ingest Rules: the stale-sequence
//! baseline and the learned slots-per-section, which must survive restarts.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sections")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub site: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub section: String,
    pub slot_count: i32,
    pub last_seq: i64,
    pub server_ts: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::slot_states::Entity")]
    SlotStates,
}

impl Related<super::slot_states::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SlotStates.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
