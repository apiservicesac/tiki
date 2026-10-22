//! Shared fixtures for the integration tests: the user's real pending work on
//! Wednesday, October 21, 2026.
//!
//! Each test file uses a different part of this module, so unused items are expected.
#![allow(dead_code)]

use chrono::NaiveDate;
use tiki_core::domain::{Project, ProjectDirectory, ProjectId, Task, TaskStatus};
use tiki_core::storage::Database;

pub struct TestDay;

impl TestDay {
    /// Wednesday, October 21, 2026.
    pub fn today() -> NaiveDate {
        Self::october(21)
    }

    pub fn october(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).expect("valid October date")
    }
}

/// A database filled with the projects and tasks from the user's list.
pub struct SampleWorkspace {
    pub database: Database,
    pub altamira: ProjectId,
    pub andeanvet: ProjectId,
    pub vapestation: ProjectId,
}

impl SampleWorkspace {
    pub fn create() -> Self {
        let database = Database::open_in_memory().expect("in-memory database");
        let projects = database.projects();
        let altamira = projects
            .insert(&Project::new("Altamira", "#2F6BD8"))
            .expect("insert");
        let andeanvet = projects
            .insert(&Project::new("Andeanvet", "#17837D"))
            .expect("insert");
        let vapestation = projects
            .insert(&Project::new("Vapestation", "#C2410C"))
            .expect("insert");

        let workspace = Self {
            database,
            altamira,
            andeanvet,
            vapestation,
        };
        workspace.insert_sample_tasks();
        workspace
    }

    pub fn directory(&self) -> ProjectDirectory {
        ProjectDirectory::new(self.database.projects().list().expect("list projects"))
    }

    fn insert_sample_tasks(&self) {
        let today = TestDay::today();
        let tasks = vec![
            Task::new("Tracking de formularios comerciales")
                .unwrap()
                .with_project(self.altamira)
                .with_due_date(TestDay::october(22)),
            Task::new("Mejora en el módulo de Ventas")
                .unwrap()
                .with_project(self.altamira)
                .with_due_date(TestDay::october(23)),
            Task::new("Acceso solo a análisis de facturas de Almendra")
                .unwrap()
                .with_project(self.andeanvet)
                .with_due_date(TestDay::october(20)),
            Self::moved(
                Task::new("Saldo vacacional con solicitud rechazada")
                    .unwrap()
                    .with_project(self.andeanvet),
                TaskStatus::InProgress,
                today,
            ),
            Self::moved(
                Task::new("Permisos de GitHub para retenciones de IGV")
                    .unwrap()
                    .with_project(self.vapestation),
                TaskStatus::Waiting,
                TestDay::october(18),
            ),
            Self::moved(
                Task::new("Confirmación de zonas del dashboard CxC")
                    .unwrap()
                    .with_project(self.andeanvet),
                TaskStatus::Waiting,
                TestDay::october(20),
            ),
            Task::new("Completar módulo de importaciones").unwrap(),
        ];
        for task in tasks {
            self.database.tasks().insert(&task).expect("insert task");
        }
    }

    fn moved(mut task: Task, status: TaskStatus, on: NaiveDate) -> Task {
        task.move_to(status, on);
        task
    }
}
