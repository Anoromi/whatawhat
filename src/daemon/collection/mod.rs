pub mod producer;
pub mod collector;
pub mod afk;

use std::sync::mpsc::Sender;


use super::storage::record_event::Record;


// struct CollectionService<C: WindowDataProducer> {
//     collector: C,
//     processor_sender: Sender<Record>
// }
//
//
// pub async fn collect() -> Result<()> {
//     Ok(())
// }


struct CollectionModule<Wpd, Ap> {
    provider: Wpd,
    afk_provider: Ap,
    sender: Sender<Record>
}

// impl<Wpd: WindowDataProvider, Ap> CollectionModule<Wpd, Ap> {}


