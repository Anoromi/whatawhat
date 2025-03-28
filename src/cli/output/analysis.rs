use std::{collections::HashMap, path::PathBuf, sync::Arc};

use chrono::Duration;

use crate::{
    cli::timeline::clean_process_name, daemon::storage::entities::UsageIntervalEntity, utils::percentage::Percentage
};

#[derive(Debug)]
pub struct ProcessUsage {
    pub process_name: Arc<str>,
    pub duration: Duration,
}

impl ProcessUsage {
    fn new(process_name: Arc<str>) -> Self {
        Self {
            process_name,
            duration: Duration::zero(),
        }
    }
}

#[derive(Debug)]
pub struct WindowUsage {
    pub process_name: Arc<str>,
    pub window_name: Arc<str>,
    pub duration: Duration,
}

impl WindowUsage {
    pub fn new(process_name: Arc<str>, window_name: Arc<str>) -> Self {
        Self {
            process_name,
            window_name,
            duration: Duration::zero(),
        }
    }
}

/// Returns vector of unique processes with their statistics + computer usage duration
pub fn analyze_processes(
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
                .entry(clean_process_name(&v.process_name))
                .or_insert_with(|| ProcessUsage::new(v.process_name));
            analysis.duration += v.duration;
        }
    }

    let threshold = interval_sum * (*min_percentage as i32) / 100;

    if !inactive.duration.is_zero() {
        map.insert(inactive.process_name.to_string(), inactive);
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
                .entry((clean_process_name(&v.process_name), v.window_name.clone()))
                .or_insert_with(|| WindowUsage::new(v.process_name, v.window_name));
            analysis.duration += v.duration;
        }
    }

    let threshold = interval_sum * (*min_percentage as i32) / 100;

    if !inactive.duration.is_zero() {
        map.insert((inactive.process_name.to_string(), inactive.window_name.clone()), inactive);
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
