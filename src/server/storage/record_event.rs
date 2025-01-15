use std::sync::Arc;

use serde::{Deserialize, Serialize};




#[derive(Serialize, Deserialize)]
pub struct RecordEventDto {
    pub timestamp: u64,
    pub proc_name: Arc<str>,
    pub wind_name: Arc<str>
}
