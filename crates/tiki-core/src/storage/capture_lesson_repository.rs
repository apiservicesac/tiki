use rusqlite::{Connection, params};

use crate::capture::CaptureLesson;
use crate::error::TikiResult;

/// Keeps the user's corrections to AI captures; only the most recent ones are kept.
pub struct CaptureLessonRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> CaptureLessonRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    /// Saves a lesson and forgets the oldest beyond `keep`.
    pub fn record(&self, lesson: &CaptureLesson, keep: usize) -> TikiResult<()> {
        self.connection.execute(
            "INSERT INTO capture_lessons (sentence, understood, corrected) VALUES (?1, ?2, ?3)",
            params![
                lesson.sentence,
                lesson.understood.to_string(),
                lesson.corrected.to_string()
            ],
        )?;
        self.connection.execute(
            "DELETE FROM capture_lessons WHERE id NOT IN
                (SELECT id FROM capture_lessons ORDER BY id DESC LIMIT ?1)",
            params![i64::try_from(keep).unwrap_or(i64::MAX)],
        )?;
        Ok(())
    }

    /// The most recent lessons, newest first.
    pub fn recent(&self, limit: usize) -> TikiResult<Vec<CaptureLesson>> {
        let mut statement = self.connection.prepare(
            "SELECT sentence, understood, corrected FROM capture_lessons ORDER BY id DESC LIMIT ?1",
        )?;
        let rows =
            statement.query_map(params![i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?;
        let mut lessons = Vec::new();
        for row in rows {
            let (sentence, understood, corrected) = row?;
            lessons.push(CaptureLesson {
                sentence,
                understood: serde_json::from_str(&understood)?,
                corrected: serde_json::from_str(&corrected)?,
            });
        }
        Ok(lessons)
    }
}
