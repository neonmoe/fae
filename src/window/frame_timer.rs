use std::thread;
use std::time::{Duration, Instant};

/// How many consecutive frames need to be within the error margin of
/// their average, so that the FPS is considered to be stable.
const STABLE_REFRESH_COUNT: usize = 20;
/// The error margin for `STABLE_REFRESH_COUNT`.
const REFRESH_DURATION_ERROR_MARGIN: Duration = Duration::from_millis(1);
/// The amount of frame durations we should store for the purposes of
/// performance monitoring.
const STORED_FRAME_TIMES: usize = 60;

pub struct FrameTimer {
    frame_durations: Vec<Duration>,
    vsync_duration: Option<Duration>,
    end: Option<Instant>,
    start: Option<Instant>,
    frame_duration: Option<Duration>,
    wait_duration: Option<Duration>,
    refresh_durations: Vec<Duration>,
}

impl FrameTimer {
    pub fn new() -> FrameTimer {
        FrameTimer {
            frame_durations: Vec::with_capacity(STORED_FRAME_TIMES + 1),
            vsync_duration: None,
            end: None,
            start: None,
            frame_duration: None,
            wait_duration: None,
            refresh_durations: Vec::with_capacity(STABLE_REFRESH_COUNT + 1),
        }
    }

    pub fn end_frame(&mut self) {
        let end = Instant::now();
        if let Some(last_end) = self.end {
            self.frame_duration = Some(end - last_end);
        }
        self.end = Some(end);
    }

    pub fn begin_frame(&mut self) {
        let (end, frame_duration, last_start) =
            if let (Some(a), Some(b), Some(c)) = (self.end, self.frame_duration, self.start) {
                (a, b, c)
            } else {
                self.start = Some(Instant::now());
                return;
            };

        // Keep a list of recent refresh durations to detect how long
        // a vsync is.
        self.refresh_durations.push(frame_duration);
        if self.refresh_durations.len() >= STABLE_REFRESH_COUNT {
            let sum = self
                .refresh_durations
                .iter()
                .fold(Duration::from_millis(0), |acc, &duration| acc + duration);
            let avg = sum / STABLE_REFRESH_COUNT as u32;

            // If the greatest difference from the average refresh
            // duration is less than a millisecond, assume that we're
            // bound by vsync (which would result in little variance),
            // and set it.
            if self
                .refresh_durations
                .iter()
                .all(|&d| if d > avg { d - avg } else { avg - d } < REFRESH_DURATION_ERROR_MARGIN)
            {
                self.vsync_duration = Some(avg - REFRESH_DURATION_ERROR_MARGIN);
            }

            self.refresh_durations.clear();
        }

        let running_time = end - last_start;
        self.frame_durations.insert(0, running_time);
        self.frame_durations.truncate(STORED_FRAME_TIMES);

        // If a vsync duration is figured out, sleep before processing
        // the frame to reduce input lag
        if let Some(vsync) = self.vsync_duration {
            if vsync > running_time {
                let wait_duration = vsync - running_time;
                self.wait_duration = Some(wait_duration);
                thread::sleep(wait_duration);
            } else {
                self.wait_duration = None;
            }
        }

        self.start = Some(Instant::now());
    }

    pub fn avg_frame_duration(&self) -> Duration {
        if self.frame_durations.len() == 0 {
            Duration::from_millis(0)
        } else {
            let sum = self
                .frame_durations
                .iter()
                .fold(Duration::from_millis(0), |acc, &duration| acc + duration);
            sum / self.frame_durations.len() as u32
        }
    }
}
