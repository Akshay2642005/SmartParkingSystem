//! `SeaORM` entities for the parking schema.
//!
//! Hand-written to match `migrations/20260827090000_parking_state.sql`;
//! regenerate them with `cargo make entity` once a database is reachable.

pub mod prelude;

pub mod sections;
pub mod slot_states;
pub mod snapshots;
