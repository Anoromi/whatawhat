use anyhow::Result;
use tokio::join;

use crate::daemon::storage::record_event::RecordEvent;

use super::module::EventProcessor;

/// Intended for future use to potentialy save to multiple backends simultaneously.
/// 
///
pub struct CombinedProcessor<A, B> {
    // receiver: Receiver<PipeEvent<Record>>,
    value: (A, B),
}

impl CombinedProcessor<(), ()> {
    pub fn new() -> Self {
        Self {
            value: ((), ()),
        }
    }
}

impl<A: EventProcessor, B: EventProcessor> EventProcessor for CombinedProcessor<A, B> {
    fn process_next(&mut self, message: RecordEvent) -> impl std::future::Future<Output = Result<()>> {
        self.value.process_next(message)
    }

    fn finalize(&mut self) -> impl std::future::Future<Output = Result<()>> {
        self.value.finalize()
    }
}

impl<A: EventProcessor, B: EventProcessor> CombinedProcessor<A, B> {
    pub fn combine<C: EventProcessor>(
        Self { value }: Self,
        next: C,
    ) -> CombinedProcessor<(A, B), C> {
        CombinedProcessor {
            value: (value, next),
        }
    }

}

impl EventProcessor for () {
    async fn process_next(&mut self, _: RecordEvent) -> Result<()> {
        Ok(())
    }

    async fn finalize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<A: EventProcessor, B: EventProcessor> EventProcessor for (A, B) {
    async fn process_next(&mut self, message: RecordEvent) -> Result<()> {
        let result = join! {
            self.0.process_next(message.clone()),
            self.1.process_next(message)
        };
        result.0?;
        result.1?;
        Ok(())
    }

    async fn finalize(&mut self) -> Result<()> {
        let result = join! {
            self.0.finalize(),
            self.1.finalize()
        };
        result.0?;
        result.1?;
        Ok(())
    }
}
