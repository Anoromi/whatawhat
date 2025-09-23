use ::serde::{Deserialize, Serialize};
use anyhow::Result;
use chrono::{serde, DateTime, Duration, Local, TimeDelta, Utc};
use now::DateTimeNow;
use specta::Type;
use tauri::State;

use crate::{
    cli::output::{
        analysis::{analyze_apps, analyze_windows, ProcessUsage, WindowUsage},
        extract_between,
        sliding_grouping::{sliding_interval_grouping, SlidingInterval, TimeOption},
        ExtractConfig,
    },
    daemon::storage::entities::UsageIntervalEntity,
    tauri::UiApp,
    utils::percentage::Percentage,
};

#[derive(Serialize, Deserialize, Type)]
pub struct UiUsageIntervalEntity {
    pub window_name: String,
    pub process_path: Option<String>,
    pub app_identifier: Option<String>,
    pub app_name: Option<String>,
    // #[serde(with = "chrono::serde::ts_seconds")]
    pub start: DateTime<Utc>,
    pub duration: Duration,
    #[serde(default)]
    pub afk: bool,
}
impl From<UsageIntervalEntity> for UiUsageIntervalEntity {
    fn from(value: UsageIntervalEntity) -> Self {
        Self {
            window_name: value.window_name.to_string(),
            process_path: value.process_path.map(|v| v.to_string()),
            app_identifier: value.app_identifier.map(|v| v.to_string()),
            app_name: value.app_name.map(|v| v.to_string()),
            start: value.start,
            duration: value.duration,
            afk: value.afk,
        }
    }
}

#[derive(Serialize, Deserialize, Type)]
pub struct GetTimelineDataParams {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    duration: u32,
    time_option: UiTimeOption,
}

#[derive(Serialize, Deserialize, Type)]
pub enum UiTimeOption {
    Hours,
    Minutes,
    Seconds,
    Days,
    Weeks,
}

impl From<UiTimeOption> for TimeOption {
    fn from(value: UiTimeOption) -> Self {
        match value {
            UiTimeOption::Hours => TimeOption::Hours,
            UiTimeOption::Minutes => TimeOption::Minutes,
            UiTimeOption::Seconds => TimeOption::Seconds,
            UiTimeOption::Days => TimeOption::Days,
            UiTimeOption::Weeks => TimeOption::Weeks,
        }
    }
}

#[derive(Serialize, Deserialize, Type)]
pub struct UiWindowUsageCell {
    pub start_time: chrono::DateTime<Utc>,
    pub data: Option<UiWindowUsageCellData>,
}

#[derive(Serialize, Deserialize, Type)]
pub struct UiWindowUsageCellData {
    pub duration: UiTimeDelta,
    pub usage: Vec<UiWindowUsage>,
}

#[derive(Serialize, Deserialize, Type)]
struct UiTimeDelta {
    secs: i64,
    nanos: i32,
}

impl From<TimeDelta> for UiTimeDelta {
    fn from(value: TimeDelta) -> Self {
        Self {
            secs: value.num_seconds(),
            nanos: value.subsec_nanos(),
        }
    }
}

impl TryFrom<UiTimeDelta> for TimeDelta {
    type Error = String;

    fn try_from(value: UiTimeDelta) -> Result<Self, Self::Error> {
        TimeDelta::new(value.secs, value.nanos as u32)
            .ok_or_else(|| "TimeDelta out of bounds".to_string())
    }
}

#[derive(Serialize, Deserialize, Type)]
pub struct UiWindowUsage {
    pub app_identifier: String,
    pub window_name: String,
    pub duration: TimeDelta,
}

impl From<WindowUsage> for UiWindowUsage {
    fn from(value: WindowUsage) -> Self {
        Self {
            app_identifier: value.app_identifier.to_string(),
            window_name: value.window_name.to_string(),
            duration: value.duration,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_timeline_window_data(
    state: State<'_, UiApp>,
    params: GetTimelineDataParams,
) -> UiAppResult<Vec<UiWindowUsageCell>> {
    let interval_stream = extract_between(
        state.record_storage.clone(),
        ExtractConfig {
            start: params.start,
            end: params.end,
        },
    );
    let min_percentage = Percentage::new_opt(10.).unwrap();
    let afk = false;
    let result = sliding_interval_grouping::<_, Local>(
        interval_stream,
        SlidingInterval::new_opt(params.duration, params.time_option.into()).unwrap(),
        |v| {
            let (usages, duration) = analyze_windows(v, min_percentage, afk);
            UiWindowUsageCellData {
                duration: duration.into(),
                usage: usages.into_iter().map(|v| UiWindowUsage::from(v)).collect(),
            }
        },
    )
    .await
    .map(|vec| {
        vec.into_iter()
            .map(|(time, data)| UiWindowUsageCell {
                start_time: time,
                data: data.map(|data| data.into()),
            })
            .collect::<Vec<_>>()
    })
    .unwrap();
    Ok(result)
}

#[derive(Serialize, Deserialize, Type)]
pub struct UiAppUsageCell {
    pub start_time: chrono::DateTime<Utc>,
    pub data: Option<UiAppUsageCellData>,
}

#[derive(Serialize, Deserialize, Type)]
pub struct UiAppUsageCellData {
    pub duration: UiTimeDelta,
    pub usage: Vec<UiAppUsage>,
}

#[derive(Serialize, Deserialize, Type)]
pub struct UiAppUsage {
    pub app_identifier: String,
    pub duration: UiTimeDelta,
}

impl From<ProcessUsage> for UiAppUsage {
    fn from(value: ProcessUsage) -> Self {
        Self {
            app_identifier: value.app_identifier.to_string(),
            duration: value.duration.into(),
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_timeline_app_data(
    state: State<'_, UiApp>,
    params: GetTimelineDataParams,
) -> UiAppResult<Vec<UiAppUsageCell>> {
    let interval_stream = extract_between(
        state.record_storage.clone(),
        ExtractConfig {
            start: params.start,
            end: params.end,
        },
    );
    let min_percentage = Percentage::new_opt(10.).unwrap();
    let afk = false;
    let result = sliding_interval_grouping::<_, Local>(
        interval_stream,
        SlidingInterval::new_opt(params.duration, params.time_option.into()).unwrap(),
        |v| {
            let (usages, duration) = analyze_apps(v, min_percentage, afk);
            UiAppUsageCellData {
                duration: duration.into(),
                usage: usages.into_iter().map(|v| UiAppUsage::from(v)).collect(),
            }
        },
    )
    .await
    .map(|vec| {
        vec.into_iter()
            .map(|(time, data)| UiAppUsageCell {
                start_time: time,
                data: data.map(|data| data.into()),
            })
            .collect::<Vec<_>>()
    })
    .unwrap();
    Ok(result)
}

pub type UiAppResult<T> = Result<T, UiAppError>;
// #[derive(Serialize, Deserialize, Type)]
// enum UiAppResult<T> {
//     Ok(T),
//     Err(UiAppError),
// }

#[derive(Serialize, Deserialize, Type)]
pub enum UiAppError {
    Other(String),
}
