-- Parking state schema (impl PHASE-29 / P5).
--
-- `sections` carries the durable ingest guards: `last_seq` is the
-- stale-sequence baseline and `slot_count` the learned slots-per-section, so
-- both survive a backend restart (architecture/communication.md § Backend
-- Ingest Rules). `snapshots` is append-only history for analytics and audit;
-- its UNIQUE (site, section, seq) makes QoS-1 redelivery a no-op at the
-- database level too.

CREATE TABLE IF NOT EXISTS sections (
    site       TEXT   NOT NULL,
    section    TEXT   NOT NULL,
    slot_count INT    NOT NULL CHECK (slot_count BETWEEN 1 AND 64),
    last_seq   BIGINT NOT NULL,
    server_ts  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (site, section)
);

CREATE TABLE IF NOT EXISTS slot_states (
    site       TEXT   NOT NULL,
    section    TEXT   NOT NULL,
    slot_id    TEXT   NOT NULL,
    state      TEXT   NOT NULL CHECK (state IN ('free', 'occupied', 'error')),
    changed_ms BIGINT NOT NULL,
    PRIMARY KEY (site, section, slot_id),
    FOREIGN KEY (site, section) REFERENCES sections (site, section) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS snapshots (
    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    site      TEXT   NOT NULL,
    section   TEXT   NOT NULL,
    seq       BIGINT NOT NULL,
    payload   JSONB  NOT NULL,
    server_ts TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (site, section, seq)
);

CREATE INDEX IF NOT EXISTS snapshots_site_section_server_ts_idx
    ON snapshots (site, section, server_ts DESC);
