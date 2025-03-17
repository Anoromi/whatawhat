use std::{fmt::Display, ops::Deref, str::FromStr};

use anyhow::anyhow;
use chrono::Duration;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Percentage(f64);

impl Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl Percentage {
    pub fn new_opt(value: f64) -> Option<Percentage> {
        if value < 0. {
            None
        } else {
            Some(Percentage(value))
        }
    }
}

impl FromStr for Percentage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This means that 100%% also works, but I think I'm fine with that
        let s = s.trim_end_matches("%");
        let v = s.parse::<f64>()?;
        Percentage::new_opt(v).ok_or_else(|| anyhow!("Can't parse {s} into percentage"))
    }
}

impl Deref for Percentage {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn duration_percentage(value: Duration, whole: Duration) -> Percentage {
    Percentage::new_opt(value.num_seconds() as f64 / whole.num_seconds() as f64 * 100.)
        .expect("Percentage should always be at least 0")
}
