use chrono::Utc;

use chrono::DateTime;
use serde::Deserialize;
use serde::Serialize;

use std::sync::Arc;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone)]
pub struct UsageIntervalEntity {
    pub window_name: Arc<str>,
    pub process_name: Arc<str>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub afk: bool,
}

impl From<UsageRecordEntity> for UsageIntervalEntity {
    fn from(
        UsageRecordEntity {
            window_name,
            process_name,
            moment,
            afk,
        }: UsageRecordEntity,
    ) -> Self {
        UsageIntervalEntity {
            window_name,
            process_name,
            start: moment,
            end: moment,
            afk,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone)]
pub struct UsageRecordEntity {
    pub window_name: Arc<str>,
    pub process_name: Arc<str>,
    pub moment: DateTime<Utc>,
    pub afk: bool,
}
