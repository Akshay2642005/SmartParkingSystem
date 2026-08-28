//! Append-only history of accepted device snapshots.
//!
//! `payload` keeps the contract JSON verbatim: debuggability of what a node
//! actually sent beats normalizing it a second time.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "snapshots")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Text")]
    pub site: String,
    #[sea_orm(column_type = "Text")]
    pub section: String,
    pub seq: i64,
    pub payload: Json,
    pub server_ts: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
