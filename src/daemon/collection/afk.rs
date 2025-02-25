pub struct AfkEvaluator {
    threshold: u32,
}

impl AfkEvaluator {
    pub fn is_afk(&self, idle_time: u32) -> bool{
        self.threshold > idle_time 
    }
}
