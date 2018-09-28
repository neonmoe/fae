use std::thread;
use std::time::{Duration, Instant};

/// How many consecutive frames need to be within the error margin of
/// their average, so that the FPS is considered to be stable.
const STABLE_FRAMES_COUNT: usize = 20;
/// The error margin for `STABLE_FRAMES_COUNT`.
const FRAME_DURATION_ERROR_MARGIN: Duration = Duration::from_millis(1);

pub struct FrameTimer {
    vsync_duration: Option<Duration>,
    end: Option<Instant>,
    start: Option<Instant>,
    frame_duration: Option<Duration>,
    wait_duration: Option<Duration>,
    frame_durations: Vec<Duration>,
}

impl FrameTimer {
    pub fn new() -> FrameTimer {
        FrameTimer {
            vsync_duration: None,
            end: None,
            start: None,
            frame_duration: None,
            wait_duration: None,
            frame_durations: Vec::with_capacity(STABLE_FRAMES_COUNT + 1),
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

        // Keep a list of recent frame durations to detect how long a
        // vsync is
        self.frame_durations.push(frame_duration);
        if self.frame_durations.len() >= STABLE_FRAMES_COUNT {
            let sum = self
                .frame_durations
                .iter()
                .fold(Duration::from_millis(0), |acc, &duration| acc + duration);
            let avg = sum / STABLE_FRAMES_COUNT as u32;

            // If the greatest difference from the average frame
            // duration is less than a millisecond, assume that we're
            // bound by vsync (which would result in little variance),
            // and set it
            if self
                .frame_durations
                .iter()
                .all(|&d| if d > avg { d - avg } else { avg - d } < FRAME_DURATION_ERROR_MARGIN)
            {
                self.vsync_duration = Some(avg - FRAME_DURATION_ERROR_MARGIN);
            }

            self.frame_durations.clear();
        }

        // If a vsync duration is figured out, sleep before processing
        // the frame to reduce input lag
        if let Some(vsync) = self.vsync_duration {
            let running_time = end - last_start;
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
}
