use std::collections::HashMap;

use crate::{
    domain::parking::{NodeStatus, decide, parse_status},
    store::{SectionStore, StatusOutcome},
};
use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, DatabaseTransaction, DbBackend, DbErr, EntityTrait, FromQueryResult, QueryFilter,
    QueryOrder, Statement, TransactionTrait,
    prelude::DateTimeWithTimeZone,
    sea_query::OnConflict,
};
use seaorm::{
    SeaOrmStore,
    store::entities::{sections, slot_states, snapshots},
};

use crate::{
    domain::parking::{Reject, SectionGuard, SectionState, parse_topic},
    protocol::{Slot, SlotState},
};

pub struct PostgresStore {
    store: SeaOrmStore,
}

impl PostgresStore {
    #[must_use]
    pub fn new(store: SeaOrmStore) -> Self {
        Self { store }
    }

    #[must_use]
    pub fn into_inner(self) -> SeaOrmStore {
        self.store
    }
}

#[derive(Debug, FromQueryResult)]
struct GuardRow {
    slot_count: i32,
    last_seq: i64,
}

fn storage(error: impl std::fmt::Display) -> Reject {
    Reject::Storage(error.to_string())
}

async fn read_guard(
    tx: &DatabaseTransaction,
    site: &str,
    section: &str,
) -> Result<Option<SectionGuard>, Reject> {
    let statment = Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELET slot_count, last_seq FROM sections WHERE site = $1 AND section = $2 FOR UPDATE",
        [site.into(), section.into()],
    );

    let row = GuardRow::find_by_statement(statment)
        .one(tx)
        .await
        .map_err(storage)?;

    Ok(row.map(|row| SectionGuard {
        last_seq: row.last_seq.max(0) as u64,
        slot_count: row.slot_count.max(0) as usize,
    }))
}

/// Write one accepted snapshot: guards, current slot state, and history.
async fn persist(
    tx: &DatabaseTransaction,
    state: &SectionState,
    payload: &[u8],
) -> Result<(), DbErr> {
    let server_ts = to_timestamp(state.server_ts_ms);

    sections::Entity::insert(sections::ActiveModel {
        site: Set(state.site.clone()),
        section: Set(state.section.clone()),
        slot_count: Set(state.slot_count as i32),
        last_seq: Set(state.seq as i64),
        server_ts: Set(server_ts),
    })
    .on_conflict(
        OnConflict::columns([sections::Column::Site, sections::Column::Section])
            .update_columns([
                sections::Column::SlotCount,
                sections::Column::LastSeq,
                sections::Column::ServerTs,
            ])
            .to_owned(),
    )
    // Nothing here needs the written row back, so skip RETURNING.
    .exec_without_returning(tx)
    .await?;

    // Snapshots are complete by contract, so the section's slot rows are
    // replaced wholesale rather than merged: no stale row can survive a slot
    // being renamed.
    slot_states::Entity::delete_many()
        .filter(slot_states::Column::Site.eq(state.site.clone()))
        .filter(slot_states::Column::Section.eq(state.section.clone()))
        .exec(tx)
        .await?;

    slot_states::Entity::insert_many(state.slots.iter().map(|slot| slot_states::ActiveModel {
        site: Set(state.site.clone()),
        section: Set(state.section.clone()),
        slot_id: Set(slot.id.clone()),
        state: Set(slot.state.as_str().to_owned()),
        changed_ms: Set(slot.changed_ms as i64),
    }))
    .exec_without_returning(tx)
    .await?;

    let history = snapshots::Entity::insert(snapshots::ActiveModel {
        id: NotSet,
        site: Set(state.site.clone()),
        section: Set(state.section.clone()),
        seq: Set(state.seq as i64),
        payload: Set(serde_json::from_slice(payload).unwrap_or(serde_json::Value::Null)),
        server_ts: Set(server_ts),
    })
    .on_conflict(
        OnConflict::columns([
            snapshots::Column::Site,
            snapshots::Column::Section,
            snapshots::Column::Seq,
        ])
        .do_nothing()
        .to_owned(),
    )
    .exec_without_returning(tx)
    .await;

    match history {
        // A redelivered snapshot is already in the history; that is the
        // intended no-op, not a failure.
        Ok(_) | Err(DbErr::RecordNotInserted) => Ok(()),
        Err(error) => Err(error),
    }
}

async fn rollback(tx: DatabaseTransaction) {
    if let Err(error) = tx.rollback().await {
        tracing::error!(%error, "failed to roll back parking state transaction");
    }
}

fn section_state(row: sections::Model, slots: Vec<Slot>) -> SectionState {
    SectionState {
        site: row.site,
        section: row.section,
        seq: row.last_seq.max(0) as u64,
        slot_count: row.slot_count.max(0) as usize,
        slots,
        server_ts_ms: row.server_ts.timestamp_millis().max(0) as u64,
    }
}

fn slot_from_row(row: slot_states::Model) -> Slot {
    Slot {
        id: row.slot_id,
        // The CHECK constraint keeps this in the protocol vocabulary; a value
        // outside it would mean the schema was edited by hand.
        state: row.state.parse().unwrap_or(SlotState::Error),
        changed_ms: row.changed_ms.max(0) as u64,
    }
}

fn to_timestamp(ms: u64) -> DateTimeWithTimeZone {
    DateTime::<Utc>::from_timestamp_millis(ms as i64)
        .unwrap_or_else(Utc::now)
        .fixed_offset()
}

#[async_trait::async_trait]
impl SectionStore for PostgresStore {
    async fn apply_status(&self, topic: &str, payload: &[u8]) -> Result<StatusOutcome, Reject> {
        let (site, section, status) = parse_status(topic, payload)?;
        if status == NodeStatus::Offline {
            sections::Entity::delete_many()
                .filter(sections::Column::Site.eq(site))
                .filter(sections::Column::Section.eq(section))
                .exec(self.store.db())
                .await
                .map_err(storage)?;
        }

        Ok(StatusOutcome {
            site: site.to_string(),
            section: section.to_string(),
            status,
        })
    }
    async fn snapshot_all(&self) -> Result<Vec<SectionState>, Reject> {
        let db = self.store.db();

        let rows = sections::Entity::find()
            .order_by_asc(sections::Column::Site)
            .order_by_asc(sections::Column::Section)
            .all(db)
            .await
            .map_err(storage)?;

        let slots = slot_states::Entity::find()
            .order_by_asc(slot_states::Column::Site)
            .order_by_asc(slot_states::Column::Section)
            .order_by_asc(slot_states::Column::SlotId)
            .all(db)
            .await
            .map_err(storage)?;

        let mut grouped: HashMap<(String, String), Vec<Slot>> = HashMap::new();

        for row in slots {
            grouped
                .entry((row.site.clone(), row.section.clone()))
                .or_default()
                .push(slot_from_row(row));
        }

        Ok(rows
            .into_iter()
            .map(|row| {
                let slots = grouped
                    .remove(&(row.site.clone(), row.section.clone()))
                    .unwrap_or_default();
                section_state(row, slots)
            })
            .collect())
    }
    async fn get(&self, site: &str, section: &str) -> Result<Option<SectionState>, Reject> {
        let db = self.store.db();

        let Some(row) = sections::Entity::find_by_id((site.to_owned(), section.to_owned()))
            .one(db)
            .await
            .map_err(storage)?
        else {
            return Ok(None);
        };

        let slots = slot_states::Entity::find()
            .filter(slot_states::Column::Site.eq(site))
            .filter(slot_states::Column::Section.eq(section))
            .all(db)
            .await
            .map_err(storage)?
            .into_iter()
            .map(slot_from_row)
            .collect();

        Ok(Some(section_state(row, slots)))
    }
    fn backend(&self) -> &'static str {
        "postgres"
    }
    async fn health(&self) -> Result<(), String> {
        self.store.ping().await.map_err(|e| e.to_string())
    }

    async fn apply_update(&self, topic: &str, payload: &[u8]) -> Result<SectionState, Reject> {
        let (site, section, _) = parse_topic(topic)?;

        let tx = self.store.db().begin().await.map_err(storage)?;

        let guard = match read_guard(&tx, site, section).await {
            Ok(guard) => guard,
            Err(error) => {
                rollback(tx).await;
                return Err(error);
            }
        };

        let state = match decide(topic, payload, guard) {
            Ok(state) => state,
            Err(reject) => {
                rollback(tx).await;
                return Err(reject);
            }
        };

        if let Err(error) = persist(&tx, &state, payload).await {
            rollback(tx).await;
            return Err(storage(error));
        }

        tx.commit().await.map_err(storage)?;
        Ok(state)
    }
}
