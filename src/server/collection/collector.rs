use anyhow::Result;
use std::rc::Rc;

struct WindowData {
    window_name: Rc<str>,
    process_name: Rc<str>,
}

pub trait Collector {
    fn get_active_window_data() -> Result<WindowData>;
}

pub struct GenericCollector {}

impl Collector for GenericCollector {
    fn get_active_window_data() -> Result<WindowData> {
        todo!()
    }
}
