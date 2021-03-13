use std::collections::VecDeque;
use std::time::{Duration, Instant};

const N_DURATIONS: usize = 360;

pub struct FrameCounter {
    absolute_count: u64,
    last_instant: Instant,
    last_frame_time: Duration,
    past_n_durations: VecDeque<Duration>,
}

impl Default for FrameCounter {
    fn default() -> Self {
        FrameCounter {
            absolute_count: 0,
            last_instant: Instant::now(),
            last_frame_time: Default::default(),
            past_n_durations: Default::default(),
        }
    }
}

impl FrameCounter {
    pub fn frame_presented(&mut self) {
        self.absolute_count += 1;
        self.last_frame_time = self.last_instant.elapsed();
        self.last_instant = Instant::now();
        self.past_n_durations.push_back(self.last_frame_time);
        while self.past_n_durations.len() > N_DURATIONS {
            self.past_n_durations.pop_front().unwrap();
        }
    }

    pub fn absolute_frame_count(&self) -> u64 {
        self.absolute_count
    }

    pub fn last_frame_time(&self) -> Duration {
        self.last_frame_time
    }

    pub fn average_frame_time(&self) -> f32 {
        self.past_n_durations
            .iter()
            .map(Duration::as_secs_f32)
            .map(|ms| ms * 1000f32)
            .sum::<f32>()
            / self.past_n_durations.len() as f32
    }

    pub fn last_fps(&self) -> f32 {
        // FPS = 1 / time to process loop
        1.0 / self.last_frame_time.as_secs_f32()
    }

    pub fn average_fps(&self) -> f32 {
        1.0 / (self
            .past_n_durations
            .iter()
            .map(Duration::as_secs_f32)
            .sum::<f32>()
            / self.past_n_durations.len() as f32)
    }

    pub fn past_n_fps(&self) -> Vec<f32> {
        self.past_n_durations
            .iter()
            .map(|d| 1.0 / d.as_secs_f32())
            .collect()
    }
}
