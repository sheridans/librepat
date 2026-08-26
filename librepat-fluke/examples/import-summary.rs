use std::{env, fs, process::ExitCode};

use librepat_core::{ImportSource, TesterImporter};
use librepat_fluke::FlukeImporter;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: import-summary <export.flk>");
        return ExitCode::FAILURE;
    };
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("could not read source: {error}");
            return ExitCode::FAILURE;
        }
    };
    match FlukeImporter::default().import(&ImportSource::new(&path, &bytes)) {
        Ok(imported) => {
            println!(
                "{} appliances, {} warnings",
                imported.job.appliances.len(),
                imported.warnings.len()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("import failed: {error}");
            ExitCode::FAILURE
        }
    }
}
