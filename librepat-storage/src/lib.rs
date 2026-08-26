//! Portable SQLite storage for LibrePAT jobs.

mod codec;
mod error;
mod insert;
mod metadata;
mod read;
mod results;
mod schema;
mod store;

pub use error::StoreError;
pub use store::{APPLICATION_ID, JobStore, SCHEMA_VERSION};
