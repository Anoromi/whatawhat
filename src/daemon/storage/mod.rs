//!  Storage is organized through [record_storage::RecordStorageImpl].
//!  The basic idea is:
//!   - There is a directory with all the records.
//!   - Records are stored using special record files, which store data for a UTC day.
//!   - Records are stored as intervals from time a, for duration b.

pub mod record_event;
pub mod record_storage;
pub mod entities;




