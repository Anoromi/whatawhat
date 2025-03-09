use anyhow::Result;

use crate::daemon::storage::record_event::RecordEvent;


pub trait EventProcessor {
    fn process_next(&mut self, message: RecordEvent) -> impl std::future::Future<Output = Result<()>>;

    fn finalize(&mut self) -> impl std::future::Future<Output = Result<()>>;
}

