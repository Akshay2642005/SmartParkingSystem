//! Current state of every slot of every section (last-write-wins).

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "slot_states")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub site: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub section: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub slot_id: String,
    /// One of `free`, `occupied`, `error` — the protocol vocabulary, enforced
    /// by a CHECK constraint in the migration.
    #[sea_orm(column_type = "Text")]
    pub state: String,
    /// Node uptime (ms) of the last observed transition; not wall clock.
    pub changed_ms: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::sections::Entity",
        from = "(Column::Site, Column::Section)",
        to = "(super::sections::Column::Site, super::sections::Column::Section)",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Sections,
}

impl Related<super::sections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sections.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
