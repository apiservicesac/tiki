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
}
