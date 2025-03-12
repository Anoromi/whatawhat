use std::{collections::HashMap, sync::Arc};

use chrono::Duration;

use crate::daemon::storage::{entities::UsageIntervalEntity, record_event::Color};

#[derive(Debug)]
pub struct UsageAnalisis {
    pub process_name: Arc<str>,
    pub duration: Duration,
    // Some day I'll have color, I'm sure of it, that's a promise
    pub color: Option<Color>,
}

impl UsageAnalisis {
    fn new(name: Arc<str>, color: Option<Color>) -> Self {
        Self {
            process_name: name,
            duration: Duration::zero(),
            color,
        }
    }
}

#[derive(Debug)]
pub struct WindowUsageAnalisis {
    pub process_name: Arc<str>,
    pub window_name: Arc<str>,
    pub duration: Duration,
    // Some day I'll have color, I'm sure of it, that's a promise
    pub color: Option<Color>,
}

impl WindowUsageAnalisis {
    pub fn new(process_anme: Arc<str>, name: Arc<str>, color: Option<Color>) -> Self {
        Self {
            process_name: process_anme,
            window_name: name,
            duration: Duration::zero(),
            color,
        }
    }
}

pub fn analyze_processes(
    intervals: Vec<UsageIntervalEntity>,
    interval_length: Duration,
    threshold: Duration,
) -> Vec<UsageAnalisis> {
    let mut map = HashMap::<Arc<str>, UsageAnalisis>::new();

    for v in intervals {
        let analysis = map
            .entry(v.process_name.clone())
            .or_insert_with(|| UsageAnalisis::new(v.process_name, None));
        analysis.duration += v.duration;
    }

    let mut inactive = UsageAnalisis::new("Inactive".into(), None);
    let overall  = map
            .iter()
            .map(|v| v.1.duration)
            .fold(Duration::zero(), |acc, next| acc + next);
    inactive.duration = interval_length
        - overall;

    map.insert(inactive.process_name.clone(), inactive);

    let mut usages = map
        .into_iter()
        .map(|v| v.1)
        .filter(|v| v.duration > threshold)
        .collect::<Vec<_>>();
    usages.sort_by(|a, b| a.duration.cmp(&b.duration));
    usages.reverse();
    usages
}

pub fn analyze_windows(
    intervals: Vec<UsageIntervalEntity>,
    threshold: Duration,
) -> Vec<WindowUsageAnalisis> {
    let mut map = HashMap::<(Arc<str>, Arc<str>), WindowUsageAnalisis>::new();

    for v in intervals {
        let analysis = map
            .entry((v.process_name.clone(), v.window_name.clone()))
            .or_insert_with(|| WindowUsageAnalisis::new(v.process_name, v.window_name, None));
        analysis.duration += v.duration;
    }
    let mut usages = map
        .into_iter()
        .map(|v| v.1)
        .filter(|v| v.duration > threshold)
        .collect::<Vec<_>>();
    usages.sort_by(|a, b| a.duration.cmp(&b.duration));
    usages.reverse();
    usages
}
