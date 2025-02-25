use anyhow::Result;
use tokio::{join, sync::mpsc::Receiver};

use crate::daemon::{pipeline_event::PipeEvent, storage::record_event::Record};

use super::module::EventProcessor;

/// Intended for future use to potentialy save to multiple backends simultaneously.
pub struct CombinedProcessor<A, B> {
    receiver: Receiver<PipeEvent<Record>>,
    value: (A, B),
}

impl CombinedProcessor<(), ()> {
    pub fn new(receiver: Receiver<PipeEvent<Record>>) -> Self {
        Self {
            value: ((), ()),
            receiver,
        }
    }
}

impl<A: EventProcessor, B: EventProcessor> EventProcessor for CombinedProcessor<A, B> {
    fn process_next(&mut self, message: Record) -> impl std::future::Future<Output = Result<()>> {
        self.value.process_next(message)
    }

    fn finalize(&mut self) -> impl std::future::Future<Output = Result<()>> {
        self.value.finalize()
    }
}

impl<A: EventProcessor, B: EventProcessor> CombinedProcessor<A, B> {
    pub fn combine<C: EventProcessor>(
        Self { receiver, value }: Self,
        next: C,
    ) -> CombinedProcessor<(A, B), C> {
        CombinedProcessor {
            value: (value, next),
            receiver,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let message = self.receiver.recv().await;

        match message {
            None | Some(PipeEvent::Close) => {
                if message.is_none() {
                    // TODO add logging for unsuccessful close
                }

                let result = self.value.finalize().await;
                self.receiver.close();
                result
            }
            Some(PipeEvent::Next(message)) => self.value.process_next(message).await,
        }
    }
}

impl EventProcessor for () {
    async fn process_next(&mut self, _: Record) -> Result<()> {
        Ok(())
    }

    async fn finalize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<A: EventProcessor, B: EventProcessor> EventProcessor for (A, B) {
    async fn process_next(&mut self, message: Record) -> Result<()> {
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
