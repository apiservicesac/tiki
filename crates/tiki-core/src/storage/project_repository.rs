use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::domain::{Project, ProjectId};
use crate::error::{TikiError, TikiResult};

/// Reads and writes projects.
pub struct ProjectRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> ProjectRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn insert(&self, project: &Project) -> TikiResult<ProjectId> {
        self.connection.execute(
            "INSERT INTO projects (name, color, notes) VALUES (?1, ?2, ?3)",
            params![project.name, project.color, project.notes],
        )?;
        Ok(ProjectId(self.connection.last_insert_rowid()))
    }

    pub fn update(&self, project: &Project) -> TikiResult<()> {
        let project_id = project.id.ok_or(TikiError::NotFound {
            entity: "project",
            id: 0,
        })?;
        let changed_rows = self.connection.execute(
            "UPDATE projects SET name = ?2, color = ?3, notes = ?4 WHERE id = ?1",
            params![project_id.0, project.name, project.color, project.notes],
        )?;
        if changed_rows == 0 {
            return Err(TikiError::NotFound {
                entity: "project",
                id: project_id.0,
            });
        }
        Ok(())
    }

    /// Removes a project; the database leaves their tasks without project.
    pub fn delete(&self, project_id: ProjectId) -> TikiResult<()> {
        self.connection
            .execute("DELETE FROM projects WHERE id = ?1", params![project_id.0])?;
        Ok(())
    }

    pub fn find_by_name(&self, name: &str) -> TikiResult<Option<Project>> {
        let project = self
            .connection
            .query_row(
                "SELECT id, name, color, notes FROM projects WHERE name = ?1 COLLATE NOCASE",
                params![name],
                Self::read_row,
            )
            .optional()?;
        Ok(project)
    }

    pub fn list(&self) -> TikiResult<Vec<Project>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name, color, notes FROM projects ORDER BY name")?;
        let projects = statement
            .query_map([], Self::read_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(projects)
    }

    fn read_row(row: &Row<'_>) -> rusqlite::Result<Project> {
        Ok(Project {
            id: Some(ProjectId(row.get("id")?)),
            name: row.get("name")?,
            color: row.get("color")?,
            notes: row.get("notes")?,
        })
    }
}
