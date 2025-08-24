use std::{collections::HashMap, sync::Arc};

use chrono::Duration;

use crate::{
    cli::timeline::clean_process_name, daemon::storage::entities::UsageIntervalEntity, utils::percentage::Percentage
};

#[derive(Debug)]
pub struct ProcessUsage {
    pub app_identifier: Arc<str>,
    pub duration: Duration,
}

impl ProcessUsage {
    fn new(app_identifier: Arc<str>) -> Self {
        Self {
            app_identifier: app_identifier,
            duration: Duration::zero(),
        }
    }
}

#[derive(Debug)]
pub struct WindowUsage {
    pub app_identifier: Arc<str>,
    pub window_name: Arc<str>,
    pub duration: Duration,
}

impl WindowUsage {
    pub fn new(app_identifier: Arc<str>, window_name: Arc<str>) -> Self {
        Self {
            app_identifier,
            window_name,
            duration: Duration::zero(),
        }
    }
}

/// Returns vector of unique processes with their statistics + computer usage duration
pub fn analyze_apps(
    intervals: Vec<UsageIntervalEntity>,
    min_percentage: Percentage,
    include_afk: bool,
) -> (Vec<ProcessUsage>, Duration) {
    let mut map = HashMap::<String, ProcessUsage>::new();

    let mut inactive = ProcessUsage::new("Inactive".into());

    let mut interval_sum = Duration::zero();

    for v in intervals {
        interval_sum += v.duration;
        if v.afk && !include_afk {
            inactive.duration += v.duration
        } else {
            let analysis = map
                .entry(clean_process_name(&v.readable_name()))
                .or_insert_with(|| ProcessUsage::new(v.readable_name().into()));
            analysis.duration += v.duration;
        }
    }

    let threshold = interval_sum * (*min_percentage as i32) / 100;

    if !inactive.duration.is_zero() {
        map.insert(inactive.app_identifier.to_string(), inactive);
    }

    let mut usages = map
        .into_iter()
        .map(|v| v.1)
        .filter(|v| v.duration > threshold)
        .collect::<Vec<_>>();
    usages.sort_by(|a, b| a.duration.cmp(&b.duration));
    usages.reverse();
    (usages, interval_sum)
}

/// Returns vector of unique windows with their statistics + computer usage duration
pub fn analyze_windows(
    intervals: Vec<UsageIntervalEntity>,
    min_percentage: Percentage,
    include_afk: bool,
) -> (Vec<WindowUsage>, Duration) {
    let mut map = HashMap::<(String, Arc<str>), WindowUsage>::new();

    let mut inactive = WindowUsage::new("Inactive".into(), "".into());

    let mut interval_sum = Duration::zero();

    for v in intervals {
        interval_sum += v.duration;
        if v.afk && !include_afk {
            inactive.duration += v.duration
        } else {
            let analysis = map
                .entry((clean_process_name(&v.readable_name()), v.window_name.clone()))
                .or_insert_with(|| WindowUsage::new(v.readable_name().into(), v.window_name.clone()));
            analysis.duration += v.duration;
        }
    }

    let threshold = interval_sum * (*min_percentage as i32) / 100;

    if !inactive.duration.is_zero() {
        map.insert((inactive.app_identifier.to_string(), inactive.window_name.clone()), inactive);
    }

    let mut usages = map
        .into_iter()
        .map(|v| v.1)
        .filter(|v| v.duration > threshold)
        .collect::<Vec<_>>();
    usages.sort_by(|a, b| a.duration.cmp(&b.duration));
    usages.reverse();
    (usages, interval_sum)
}
