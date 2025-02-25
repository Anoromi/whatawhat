use anyhow::Result;

use crate::daemon::storage::record_event::Record;


pub trait EventProcessor {
    fn process_next(&mut self, message: Record) -> impl std::future::Future<Output = Result<()>>;

    fn finalize(&mut self) -> impl std::future::Future<Output = Result<()>>;
}

