use std::{fmt::Display, ops::Deref, str::FromStr};

use anyhow::anyhow;


#[derive(Debug, Clone, Copy)]
pub struct Percentage(f32);


impl Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}


impl Percentage {
    pub fn new_opt(value: f32) -> Option<Percentage> {
        if value < 0.0 {
            None
        }
        else {
            Some(Percentage(value))
        }
    }
}

impl FromStr for Percentage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This means than 100%% also works, but I think I'm fine with that
        let s = s.trim_end_matches("%");
        let v = s.parse::<f32>()?;
        Percentage::new_opt(v).ok_or_else(|| anyhow!("Can't parse {s} into percentage"))
    }

}

impl Deref for Percentage {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


