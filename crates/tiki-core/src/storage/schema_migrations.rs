use rusqlite_migration::{M, Migrations};

/// Every schema change, in order. Never edit a published step: add a new one.
pub struct SchemaMigrations;

impl SchemaMigrations {
    pub fn all() -> Migrations<'static> {
        Migrations::new(vec![
            M::up(Self::INITIAL_SCHEMA),
            M::up(Self::PREFERENCES),
            M::up(Self::REMINDER_LOG),
            M::up(Self::DELIVERY_AND_TASK_REMINDERS),
            M::up(Self::CLIENTS_BECOME_PROJECTS),
            M::up(Self::KINDS_TAGS_NOTES_AND_LESSONS),
            M::up(Self::GENERAL_STATUSES_REPEATS_AND_FOLLOW_UPS),
            M::up(Self::CALENDARS),
            M::up(Self::TIMELESS_REMINDERS_BECOME_TASKS),
            M::up(Self::FOLLOW_UPS_APART_FROM_REMINDERS),
            M::up(Self::ITEM_EVENTS),
            M::up(Self::TRASH),
            M::up(Self::SESSIONS),
            M::up(Self::ACCOUNT_IDENTITY),
        ])
    }

    /// The name and small picture of each calendar account, and when they were last asked.
    const ACCOUNT_IDENTITY: &'static str = "
        ALTER TABLE calendar_accounts ADD COLUMN display_name TEXT NOT NULL DEFAULT '';
        ALTER TABLE calendar_accounts ADD COLUMN avatar TEXT;
        ALTER TABLE calendar_accounts ADD COLUMN identity_checked_at TEXT;
    ";

    /// Recorded sessions: the session, its transcript (searchable with FTS5, accents
    /// ignored), the AI notes and the tasks they propose, and the transcription queue. A
    /// task created from a session points back to it.
    const SESSIONS: &'static str = "
        CREATE TABLE sessions (
            id                INTEGER PRIMARY KEY,
            title             TEXT NOT NULL,
            mode              TEXT NOT NULL,
            status            TEXT NOT NULL,
            started_at        TEXT NOT NULL,
            paused_ms         INTEGER NOT NULL DEFAULT 0,
            paused_since      TEXT,
            ended_at          TEXT,
            media_file        TEXT,
            media_bytes       INTEGER NOT NULL DEFAULT 0,
            media_deleted_at  TEXT,
            meeting_title     TEXT,
            meeting_starts_at TEXT,
            language          TEXT,
            failure           TEXT
        );

        CREATE INDEX sessions_by_start ON sessions(started_at);

        CREATE TABLE transcript_segments (
            id         INTEGER PRIMARY KEY,
            session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            speaker    TEXT NOT NULL,
            start_ms   INTEGER NOT NULL,
            end_ms     INTEGER NOT NULL,
            text       TEXT NOT NULL
        );

        CREATE INDEX transcript_segments_by_session ON transcript_segments(session_id, start_ms);

        CREATE VIRTUAL TABLE transcript_search USING fts5(
            text, content = 'transcript_segments', content_rowid = 'id',
            tokenize = 'unicode61 remove_diacritics 2'
        );

        CREATE TRIGGER transcript_segments_indexed AFTER INSERT ON transcript_segments BEGIN
            INSERT INTO transcript_search (rowid, text) VALUES (new.id, new.text);
        END;

        CREATE TRIGGER transcript_segments_unindexed AFTER DELETE ON transcript_segments BEGIN
            INSERT INTO transcript_search (transcript_search, rowid, text)
            VALUES ('delete', old.id, old.text);
        END;

        CREATE TABLE session_notes (
            session_id INTEGER PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
            summary    TEXT NOT NULL,
            key_points TEXT NOT NULL DEFAULT '[]',
            decisions  TEXT NOT NULL DEFAULT '[]',
            written_at TEXT NOT NULL
        );

        CREATE TABLE session_action_items (
            id         INTEGER PRIMARY KEY,
            session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            title      TEXT NOT NULL,
            due_date   TEXT,
            task_id    INTEGER REFERENCES tasks(id) ON DELETE SET NULL
        );

        CREATE INDEX session_action_items_by_session ON session_action_items(session_id);

        CREATE TABLE transcription_jobs (
            session_id INTEGER PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
            queued_at  TEXT NOT NULL,
            attempts   INTEGER NOT NULL DEFAULT 0
        );

        ALTER TABLE tasks ADD COLUMN origin_session_id INTEGER
            REFERENCES sessions(id) ON DELETE SET NULL;
    ";

    /// When the user sent an item to the trash (local time); NULL for every item outside it.
    const TRASH: &'static str = "
        ALTER TABLE tasks ADD COLUMN trashed_at TEXT;
        CREATE INDEX tasks_by_trash ON tasks(trashed_at) WHERE trashed_at IS NOT NULL;
    ";

    const INITIAL_SCHEMA: &'static str = "
        CREATE TABLE clients (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL
        );

        CREATE TABLE tasks (
            id                INTEGER PRIMARY KEY,
            title             TEXT NOT NULL,
            notes             TEXT NOT NULL DEFAULT '',
            client_id         INTEGER REFERENCES clients(id) ON DELETE SET NULL,
            status            TEXT NOT NULL,
            priority          TEXT NOT NULL,
            due_date          TEXT,
            waiting_since     TEXT,
            origin_plugin     TEXT,
            origin_connection TEXT,
            origin_record     TEXT,
            origin_url        TEXT,
            created_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX tasks_by_status ON tasks(status);

        CREATE UNIQUE INDEX tasks_by_external_record
            ON tasks(origin_plugin, origin_connection, origin_record)
            WHERE origin_plugin IS NOT NULL;
    ";

    /// The activity history: one row per thing that happened to an item, at local time.
    /// Deliveries made before it existed are recorded at the start of their delivery day,
    /// oldest first, so the history keeps the order it had. A reminder that rang is not a
    /// delivery, so it gets none.
    const ITEM_EVENTS: &'static str = "
        CREATE TABLE item_events (
            id      INTEGER PRIMARY KEY,
            item_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            at      TEXT NOT NULL,
            kind    TEXT NOT NULL,
            payload TEXT NOT NULL DEFAULT '{}'
        );

        CREATE INDEX item_events_by_item ON item_events(item_id, at);
        CREATE INDEX item_events_by_kind ON item_events(kind, at);

        INSERT INTO item_events (item_id, at, kind)
        SELECT id, completed_on || ' 00:00:00', 'delivered' FROM tasks
         WHERE status = 'done' AND completed_on IS NOT NULL AND kind = 'task'
         ORDER BY completed_on, updated_at, id;
    ";

    /// The automatic follow-up nudge gets its own column instead of sharing `remind_at` with
    /// the user's reminders. A waiting item's `remind_at` moves over only when it is exactly
    /// the nudge Tiki scheduled: 3 days after it started waiting, at 09:00 (the settings every
    /// install had until now). Any other time was set by the user and stays a reminder.
    const FOLLOW_UPS_APART_FROM_REMINDERS: &'static str = "
        ALTER TABLE tasks ADD COLUMN follow_up_at TEXT;
        UPDATE tasks SET follow_up_at = remind_at, remind_at = NULL
         WHERE status = 'waiting' AND waiting_since IS NOT NULL
           AND remind_at = date(waiting_since, '+3 days') || ' 09:00:00';
        CREATE INDEX tasks_by_follow_up ON tasks(follow_up_at) WHERE follow_up_at IS NOT NULL;
    ";

    /// An open reminder without a time showed in no view; as a task it shows in the Tasks tab.
    const TIMELESS_REMINDERS_BECOME_TASKS: &'static str = "
        UPDATE tasks SET kind = 'task', repeat = 'never'
         WHERE kind = 'reminder' AND remind_at IS NULL AND status <> 'done';
    ";

    /// Connected calendar accounts and the local copy of their meetings.
    const CALENDARS: &'static str = "
        CREATE TABLE calendar_accounts (
            id       INTEGER PRIMARY KEY,
            provider TEXT NOT NULL,
            email    TEXT NOT NULL,
            UNIQUE (provider, email)
        );

        CREATE TABLE meetings (
            id          INTEGER PRIMARY KEY,
            account_id  INTEGER NOT NULL REFERENCES calendar_accounts(id) ON DELETE CASCADE,
            external_id TEXT NOT NULL,
            title       TEXT NOT NULL,
            starts_at   TEXT NOT NULL,
            ends_at     TEXT NOT NULL,
            all_day     INTEGER NOT NULL DEFAULT 0,
            location    TEXT NOT NULL DEFAULT '',
            join_url    TEXT,
            alerted     INTEGER NOT NULL DEFAULT 0,
            UNIQUE (account_id, external_id)
        );

        CREATE INDEX meetings_by_start ON meetings(starts_at);
    ";

    /// Four general statuses instead of a consultant's five, repeating reminders, and what a
    /// waiting task waits for (free text: Tiki keeps no list of people).
    const GENERAL_STATUSES_REPEATS_AND_FOLLOW_UPS: &'static str = "
        UPDATE tasks SET status = 'in_progress' WHERE status = 'testing';
        UPDATE tasks SET status = 'waiting' WHERE status = 'waiting_on_others';
        ALTER TABLE tasks ADD COLUMN repeat TEXT NOT NULL DEFAULT 'never';
        ALTER TABLE tasks ADD COLUMN waiting_for TEXT NOT NULL DEFAULT '';
    ";

    /// Standalone reminders next to tasks, tags across projects, notes per project, and the
    /// corrections the user made to what the AI understood.
    const KINDS_TAGS_NOTES_AND_LESSONS: &'static str = "
        ALTER TABLE tasks ADD COLUMN kind TEXT NOT NULL DEFAULT 'task';
        ALTER TABLE tasks ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';
        ALTER TABLE projects ADD COLUMN notes TEXT NOT NULL DEFAULT '';

        CREATE TABLE capture_lessons (
            id         INTEGER PRIMARY KEY,
            sentence   TEXT NOT NULL,
            understood TEXT NOT NULL,
            corrected  TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
    ";

    /// Tiki groups work by project (a client, a product, a team or personal), not by client.
    const CLIENTS_BECOME_PROJECTS: &'static str = "
        ALTER TABLE clients RENAME TO projects;
        ALTER TABLE tasks RENAME COLUMN client_id TO project_id;
    ";

    const DELIVERY_AND_TASK_REMINDERS: &'static str = "
        ALTER TABLE tasks ADD COLUMN completed_on TEXT;
        ALTER TABLE tasks ADD COLUMN remind_at TEXT;
        UPDATE tasks SET completed_on = date(updated_at) WHERE status = 'done';
        CREATE INDEX tasks_by_completion ON tasks(completed_on) WHERE status = 'done';
        CREATE INDEX tasks_by_reminder ON tasks(remind_at) WHERE remind_at IS NOT NULL;
    ";

    const REMINDER_LOG: &'static str = "
        CREATE TABLE reminder_log (
            moment  TEXT PRIMARY KEY,
            sent_on TEXT NOT NULL
        );
    ";

    const PREFERENCES: &'static str = "
        CREATE TABLE preferences (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
    ";
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::SchemaMigrations;

    /// A database migrated up to (and including) the step at `index`, counted from zero.
    fn migrated_through(index: usize) -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        SchemaMigrations::all()
            .to_version(&mut connection, index + 1)
            .unwrap();
        connection
    }

    fn finish(connection: &mut Connection) {
        SchemaMigrations::all().to_latest(connection).unwrap();
    }

    fn kind_of(connection: &Connection, title: &str) -> String {
        connection
            .query_row("SELECT kind FROM tasks WHERE title = ?1", [title], |row| {
                row.get(0)
            })
            .unwrap()
    }

    #[test]
    fn open_reminders_without_a_time_become_tasks() {
        let mut connection = migrated_through(7);
        connection
            .execute_batch(
                "INSERT INTO tasks (title, status, priority, kind, remind_at) VALUES
                    ('Sin hora', 'to_do', 'normal', 'reminder', NULL),
                    ('Con hora', 'to_do', 'normal', 'reminder', '2026-10-21 17:00:00'),
                    ('Ya sonó', 'done', 'normal', 'reminder', NULL);",
            )
            .unwrap();

        finish(&mut connection);

        assert_eq!(kind_of(&connection, "Sin hora"), "task");
        assert_eq!(kind_of(&connection, "Con hora"), "reminder");
        assert_eq!(kind_of(&connection, "Ya sonó"), "reminder");
    }

    fn reminder_columns(connection: &Connection, title: &str) -> (Option<String>, Option<String>) {
        connection
            .query_row(
                "SELECT remind_at, follow_up_at FROM tasks WHERE title = ?1",
                [title],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap()
    }

    #[test]
    fn only_the_automatic_nudge_of_a_waiting_item_becomes_a_follow_up() {
        use rusqlite::ToSql;
        use rusqlite::types::{ToSqlOutput, Value};
        // The backfill compares text, so it must match how a moment is written.
        let nudge_moment = chrono::NaiveDate::from_ymd_opt(2026, 10, 24)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap();
        assert_eq!(
            nudge_moment.to_sql().unwrap(),
            ToSqlOutput::Owned(Value::Text("2026-10-24 09:00:00".into()))
        );
        let mut connection = migrated_through(8);
        connection
            .execute_batch(
                "INSERT INTO tasks (title, status, priority, waiting_since, remind_at) VALUES
                    ('Automático', 'waiting', 'normal', '2026-10-21', '2026-10-24 09:00:00'),
                    ('Del usuario', 'waiting', 'normal', '2026-10-21', '2026-10-24 16:00:00'),
                    ('Ya no espera', 'to_do', 'normal', NULL, '2026-10-24 09:00:00');",
            )
            .unwrap();

        finish(&mut connection);

        let nudge = Some("2026-10-24 09:00:00".to_owned());
        assert_eq!(
            reminder_columns(&connection, "Automático"),
            (None, nudge.clone())
        );
        assert_eq!(
            reminder_columns(&connection, "Del usuario"),
            (Some("2026-10-24 16:00:00".to_owned()), None)
        );
        assert_eq!(reminder_columns(&connection, "Ya no espera"), (nudge, None));
    }

    #[test]
    fn deliveries_made_before_the_history_are_recorded_in_their_order() {
        let mut connection = migrated_through(9);
        connection
            .execute_batch(
                "INSERT INTO tasks (title, status, priority, completed_on, updated_at) VALUES
                    ('Segunda', 'done', 'normal', '2026-10-20', '2026-10-20 18:00:00'),
                    ('Primera', 'done', 'normal', '2026-10-20', '2026-10-20 10:00:00'),
                    ('Abierta', 'to_do', 'normal', NULL, '2026-10-20 10:00:00');
                 INSERT INTO tasks (title, kind, status, priority, completed_on, updated_at) VALUES
                    ('Sonó', 'reminder', 'done', 'normal', '2026-10-20', '2026-10-20 09:00:00');",
            )
            .unwrap();

        finish(&mut connection);

        let mut statement = connection
            .prepare(
                "SELECT t.title, e.at, e.kind, e.payload FROM item_events e
                   JOIN tasks t ON t.id = e.item_id ORDER BY e.id",
            )
            .unwrap();
        let events: Vec<(String, String, String, String)> = statement
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let delivery = |title: &str| {
            (
                title.to_owned(),
                "2026-10-20 00:00:00".to_owned(),
                "delivered".to_owned(),
                "{}".to_owned(),
            )
        };
        assert_eq!(events, [delivery("Primera"), delivery("Segunda")]);
    }

    #[test]
    fn accounts_connected_before_identities_have_no_name_nor_picture_yet() {
        let mut connection = migrated_through(12);
        connection
            .execute(
                "INSERT INTO calendar_accounts (provider, email) VALUES ('google', 'ana@acme.com')",
                [],
            )
            .unwrap();

        finish(&mut connection);

        let identity: (String, Option<String>, Option<String>) = connection
            .query_row(
                "SELECT display_name, avatar, identity_checked_at FROM calendar_accounts",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(identity, (String::new(), None, None));
    }
}
