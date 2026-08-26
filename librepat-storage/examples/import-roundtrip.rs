use std::{env, fs, process::ExitCode};

use librepat_core::{ImportSource, TesterImporter};
use librepat_fluke::FlukeImporter;
use librepat_storage::JobStore;

fn main() -> ExitCode {
    let Some(source_path) = env::args().nth(1) else {
        eprintln!("usage: import-roundtrip <export.flk>");
        return ExitCode::FAILURE;
    };
    match validate(&source_path) {
        Ok(count) => {
            println!("{count} appliances round-tripped without changes");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("validation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn validate(source_path: &str) -> Result<usize, String> {
    let bytes = fs::read(source_path).map_err(|error| error.to_string())?;
    let filename = std::path::Path::new(source_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("import.flk");
    let imported = FlukeImporter::default()
        .import(&ImportSource::new(filename, &bytes))
        .map_err(|error| error.to_string())?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let destination = env::temp_dir().join(format!(
        "librepat-roundtrip-{}-{nonce}.librepat",
        std::process::id()
    ));
    if destination.exists() {
        fs::remove_file(&destination).map_err(|error| error.to_string())?;
    }
    JobStore::create(&destination, &imported.job).map_err(|error| error.to_string())?;
    let reopened = JobStore::open(&destination)
        .and_then(|store| store.load())
        .map_err(|error| error.to_string())?;
    let _result = fs::remove_file(destination);
    if reopened != imported.job {
        return Err("reopened job differs from imported job".into());
    }
    Ok(reopened.appliances.len())
}
