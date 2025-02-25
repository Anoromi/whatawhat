use std::sync::Arc;

use crate::daemon::storage::record_event::Color;

#[derive(Clone)]
pub struct WindowData {
    pub window_title: Arc<str>,
    pub process_name: Arc<str>,
    pub color: Option<Color>,
    pub afk: bool
}
//
// pub trait WindowDataProvider {
//     fn get_active_window_data(&self) -> Result<WindowData>;
// }
//
// pub struct GenericWindowDataProducer {}
//
// impl WindowDataProvider for GenericWindowDataProducer {
//     fn get_active_window_data(&self) -> Result<WindowData> {
//         todo!()
//         // get_active()
//     }
// }
//
// pub trait AfkProvider {
//     fn is_afk(&self) -> Result<bool>;
// }
//
// pub struct GenericAfkProvider {
//
// }
//
// impl AfkProvider for GenericAfkProvider {
//     fn is_afk(&self) -> Result<bool> {
//         let idle_time = get_idle_time()?;
//         if idle_time / 1000 > 30 {
//
//         }
//         todo!()
//     }
// }
