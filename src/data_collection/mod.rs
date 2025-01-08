use std::{
    sync::{self, mpsc::Sender},
    time::Instant,
};

struct ActiveProcessData {
    window_name: String,
    process_path: String,
    timestamp: Instant,
}

trait DataCollector {
    fn collect() -> ActiveProcessData;
}

trait CollectionService {
    fn start(sender: &Sender<ActiveProcessData>) -> impl CollectionServiceHandle;
}

trait CollectionServiceHandle {
}



fn start_data_collection() {
    // get api key for cennection
    // 

    // let (s, r) = sync::mpsc::channel::<ActiveProcessData>();
    // let se = &s;
    // se.send(t)
}
