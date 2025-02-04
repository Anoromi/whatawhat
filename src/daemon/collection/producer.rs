use anyhow::Result;
use std::{rc::Rc, sync::Arc};

use crate::daemon::storage::record_event::Color;

#[derive(Clone)]
pub struct WindowData {
    pub window_name: Arc<str>,
    pub process_name: Arc<str>,
    pub color: Option<Color>,
}

pub trait WindowDataProvider {
    fn get_active_window_data(&self) -> Result<WindowData>;
}

pub struct GenericWindowDataProducer {}

impl WindowDataProvider for GenericWindowDataProducer {
    fn get_active_window_data(&self) -> Result<WindowData> {
        todo!()
        // get_active()
    }
}

pub trait AfkProvider {
    fn is_afk(&self) -> Result<WindowData>;
}
