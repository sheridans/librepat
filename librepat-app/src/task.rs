use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        mpsc::{self, Receiver, TryRecvError},
    },
    thread,
};

use librepat_core::{ImportSource, ImportWarning, ImporterMetadata, ImporterRegistry, Job};
use librepat_report::{generate_appliance_report, generate_completion_certificate, save_pdf};
use librepat_storage::JobStore;

#[derive(Clone, Copy)]
pub(crate) enum ReportKind {
    Appliance,
    Certificate,
}

pub(crate) enum WorkerResult {
    Imported {
        job: Box<Job>,
        warnings: Vec<ImportWarning>,
        destination: JobDestination,
    },
    Created(PathBuf),
    Reported(PathBuf),
    Failed(String),
}

pub(crate) struct JobDestination {
    pub(crate) path: PathBuf,
    pub(crate) replace_existing: bool,
}

impl JobDestination {
    pub(crate) const fn new(path: PathBuf, replace_existing: bool) -> Self {
        Self {
            path,
            replace_existing,
        }
    }
}

pub(crate) struct TaskHub {
    importers: Arc<ImporterRegistry>,
    receiver: Option<Receiver<WorkerResult>>,
    label: Option<String>,
}

impl TaskHub {
    pub(crate) fn new(importers: ImporterRegistry) -> Self {
        Self {
            importers: Arc::new(importers),
            receiver: None,
            label: None,
        }
    }

    pub fn is_busy(&self) -> bool {
        self.receiver.is_some()
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub(crate) fn importer_metadata(&self) -> Vec<ImporterMetadata> {
        self.importers.metadata().collect()
    }

    pub fn import(&mut self, source: PathBuf, destination: JobDestination) {
        let importers = Arc::clone(&self.importers);
        self.spawn("Importing tester export", move || {
            let bytes =
                fs::read(&source).map_err(|error| format!("Could not read export: {error}"))?;
            let filename = source
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("tester-export");
            let imported = importers
                .import(&ImportSource::new(filename, &bytes))
                .map_err(|error| format!("Import failed: {error}"))?;
            Ok(WorkerResult::Imported {
                job: Box::new(imported.job),
                warnings: imported.warnings,
                destination,
            })
        });
    }

    pub fn create(&mut self, job: Job, destination: JobDestination) {
        let label = if destination.replace_existing {
            "Replacing portable job"
        } else {
            "Creating portable job"
        };
        self.spawn(label, move || {
            let result = if destination.replace_existing {
                JobStore::replace(&destination.path, &job)
            } else {
                JobStore::create(&destination.path, &job)
            };
            result.map_err(|error| format!("Could not create job: {error}"))?;
            Ok(WorkerResult::Created(destination.path))
        });
    }

    pub fn report(&mut self, job: Job, destination: PathBuf, kind: ReportKind) {
        self.spawn("Generating PDF", move || {
            let bytes = match kind {
                ReportKind::Appliance => generate_appliance_report(&job),
                ReportKind::Certificate => generate_completion_certificate(&job),
            }
            .map_err(|error| format!("Could not generate PDF: {error}"))?;
            save_pdf(&destination, &bytes)
                .map_err(|error| format!("Could not save PDF: {error}"))?;
            Ok(WorkerResult::Reported(destination))
        });
    }

    pub fn receive(&mut self) -> Option<WorkerResult> {
        match self.receiver.as_ref()?.try_recv() {
            Ok(result) => {
                self.receiver = None;
                self.label = None;
                Some(result)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.receiver = None;
                let label = self
                    .label
                    .take()
                    .unwrap_or_else(|| "Background task".into());
                Some(WorkerResult::Failed(format!(
                    "{label} stopped before returning a result"
                )))
            }
        }
    }

    fn spawn(
        &mut self,
        label: &str,
        operation: impl FnOnce() -> Result<WorkerResult, String> + Send + 'static,
    ) {
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.label = Some(label.to_owned());
        thread::spawn(move || {
            let result = operation().unwrap_or_else(WorkerResult::Failed);
            let _result = sender.send(result);
        });
    }
}
