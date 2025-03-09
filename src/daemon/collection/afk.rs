pub struct AfkEvaluator {
    threshold_ms: u32,
}

impl AfkEvaluator {
    pub fn from_seconds(threshold_s: u32) -> Self {
        Self { threshold_ms: threshold_s * 1000 }
    }

    pub fn is_afk(&self, idle_time: u32) -> bool{
        self.threshold_ms < idle_time 
    }
}
