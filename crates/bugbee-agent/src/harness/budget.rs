use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// OpenCode doom-loop threshold analogue.
pub const DOOM_LOOP_THRESHOLD: usize = 3;

#[derive(Debug, Clone)]
pub struct RunLimits {
    pub max_steps: u32,
    pub max_tool_calls: u32,
    pub max_wall: Duration,
    pub max_output_chars: usize,
}

impl Default for RunLimits {
    fn default() -> Self {
        Self {
            max_steps: 24,
            max_tool_calls: 64,
            max_wall: Duration::from_secs(300),
            max_output_chars: 24_000,
        }
    }
}

impl RunLimits {
    pub fn aggressive() -> Self {
        Self {
            max_steps: 32,
            max_tool_calls: 96,
            max_wall: Duration::from_secs(600),
            max_output_chars: 32_000,
        }
    }

    pub fn scout() -> Self {
        Self {
            max_steps: 12,
            max_tool_calls: 32,
            max_wall: Duration::from_secs(120),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct Budget {
    pub limits: RunLimits,
    pub steps: u32,
    pub tool_calls: u32,
    started: Instant,
    recent_sigs: VecDeque<String>,
}

impl Budget {
    pub fn new(limits: RunLimits) -> Self {
        Self {
            limits,
            steps: 0,
            tool_calls: 0,
            started: Instant::now(),
            recent_sigs: VecDeque::with_capacity(DOOM_LOOP_THRESHOLD + 1),
        }
    }

    pub fn tick_step(&mut self) -> Result<(), String> {
        self.steps += 1;
        if self.steps > self.limits.max_steps {
            return Err(format!(
                "max steps exceeded ({}/{})",
                self.steps, self.limits.max_steps
            ));
        }
        if self.started.elapsed() > self.limits.max_wall {
            return Err(format!(
                "wall clock budget exceeded ({:?})",
                self.limits.max_wall
            ));
        }
        Ok(())
    }

    pub fn tick_tools(&mut self, n: u32) -> Result<(), String> {
        self.tool_calls += n;
        if self.tool_calls > self.limits.max_tool_calls {
            return Err(format!(
                "max tool calls exceeded ({}/{})",
                self.tool_calls, self.limits.max_tool_calls
            ));
        }
        Ok(())
    }

    /// Detect repeated identical tool call signatures (doom loop).
    pub fn note_signature(&mut self, sig: String) -> bool {
        self.recent_sigs.push_back(sig.clone());
        if self.recent_sigs.len() > DOOM_LOOP_THRESHOLD {
            self.recent_sigs.pop_front();
        }
        if self.recent_sigs.len() >= DOOM_LOOP_THRESHOLD
            && self.recent_sigs.iter().all(|s| s == &sig)
        {
            return true; // doom loop
        }
        false
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}
