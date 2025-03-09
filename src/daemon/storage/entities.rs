use chrono::Duration;
use chrono::Utc;

use chrono::DateTime;
use serde::Deserialize;
use serde::Serialize;

use std::sync::Arc;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone)]
pub struct UsageIntervalEntity {
    pub window_name: Arc<str>,
    pub process_name: Arc<str>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start: DateTime<Utc>,
    #[serde(with = "duration_ser")]
    pub duration: Duration,
    #[serde(default)]
    pub afk: bool,
}

impl UsageIntervalEntity {
    pub fn end(&self) -> DateTime<Utc> {
        self.start + self.duration
    }

    pub fn set_end(&mut self, v: DateTime<Utc>) {
        self.duration = v - self.start;
    }

    pub fn split_by(
        self,
        split: DateTime<Utc>,
    ) -> (Option<UsageIntervalEntity>, Option<UsageIntervalEntity>) {
        let end = self.end();
        if split < self.start {
            (None, Some(self))
        } else if split >= end {
            (Some(self), None)
        } else {
            let before = UsageIntervalEntity {
                window_name: self.window_name.clone(),
                process_name: self.process_name.clone(),
                start: self.start,
                duration: split - self.start,
                afk: self.afk,
            };
            let after = UsageIntervalEntity {
                window_name: self.window_name,
                process_name: self.process_name,
                start: split,
                duration: end - split,
                afk: self.afk,
            };
            (Some(before), Some(after))
        }
    }

    pub fn filter_by_interval(
        self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Option<UsageIntervalEntity> {
        self.split_by(from).1.and_then(|v| v.split_by(to).0)
    }

    pub fn with_duration(self, duration: Duration) -> Self {
        Self { duration, ..self }
    }
}

mod duration_ser {
    use chrono::Duration;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(duration.num_seconds())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = i64::deserialize(deserializer)?;
        let duration = Duration::seconds(s);
        Ok(duration)
    }
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
            duration: Duration::zero(),
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
