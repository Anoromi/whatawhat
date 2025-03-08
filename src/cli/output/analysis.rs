use std::{collections::HashMap, sync::Arc};

use chrono::Duration;

use crate::daemon::storage::{entities::UsageIntervalEntity, record_event::Color};

#[derive(Debug)]
pub struct UsageAnalisis {
    pub duration: Duration,
    // percentage: u16,
    pub name: Arc<str>,
    // Some day I'll have color, I'm sure of it, that's a promise
    pub color: Option<Color>,
}

impl UsageAnalisis {
    fn new(name: Arc<str>, color: Option<Color>) -> Self {
        Self {
            duration: Duration::zero(),
            name,
            color,
        }
    }
}



pub fn analyze_processes(intervals: Vec<UsageIntervalEntity>, threshold: Duration) -> Vec<UsageAnalisis> {
    let mut map = HashMap::<Arc<str>, UsageAnalisis>::new();

    for v in intervals {
        let analysis = map
            .entry(v.process_name.clone())
            .or_insert_with(|| UsageAnalisis::new(v.process_name, None));
        analysis.duration += v.duration;
    }
    let mut usages = map.into_iter().map(|v| v.1).filter(|v| v.duration > threshold).collect::<Vec<_>>();
    usages.sort_by(|a, b| a.duration.cmp(&b.duration));
    usages.reverse();
    usages
}
