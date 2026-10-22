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
        ])
    }

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
