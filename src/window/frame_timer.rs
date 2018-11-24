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
    clear_duration: Option<Duration>,
    clear_start: Option<Instant>,

    frame_duration: Option<Duration>,
    wait_duration: Option<Duration>,
    refresh_durations: Vec<Duration>,

    fps_counter: u32,
    fps: u32,
    last_fps_update: Instant,
}

impl FrameTimer {
    pub(crate) fn new() -> FrameTimer {
        FrameTimer {
            frame_durations: Vec::with_capacity(STORED_FRAME_TIMES + 1),
            vsync_duration: None,
            end: None,
            start: None,
            clear_duration: None,
            clear_start: None,
            frame_duration: None,
            wait_duration: None,
            refresh_durations: Vec::with_capacity(STABLE_REFRESH_COUNT + 1),
            fps_counter: 0,
            fps: 0,
            last_fps_update: Instant::now(),
        }
    }

    pub(crate) fn start_clear(&mut self) {
        self.clear_start = Some(Instant::now());
    }

    pub(crate) fn end_clear(&mut self) {
        if let Some(last_clear_start) = self.clear_start {
            self.clear_duration = Some(Instant::now() - last_clear_start);
        }
    }

    pub(crate) fn end_frame(&mut self) {
        let end = Instant::now();
        if let (Some(last_end), Some(last_clear_duration)) = (self.end, self.clear_duration) {
            if end > last_end + last_clear_duration {
                self.frame_duration = Some(end - (last_end + last_clear_duration));
            }
        }
        self.end = Some(end);
    }

    pub(crate) fn begin_frame(&mut self) {
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
            if avg > REFRESH_DURATION_ERROR_MARGIN
                && self.refresh_durations.iter().all(
                    |&d| if d > avg { d - avg } else { avg - d } < REFRESH_DURATION_ERROR_MARGIN,
                ) {
                self.vsync_duration = Some(avg - REFRESH_DURATION_ERROR_MARGIN);
            }

            self.refresh_durations.clear();
        }

        let running_time = if end > last_start {
            end - last_start
        } else {
            Duration::from_millis(0)
        };
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

        self.fps_counter += 1;
        if Instant::now() - self.last_fps_update >= Duration::from_secs(1) {
            self.fps = self.fps_counter;
            self.last_fps_update = Instant::now();
            self.fps_counter = 0;
        }

        self.start = Some(Instant::now());
    }

    /// Returns the average duration of the last 60 frames. A "frame"
    /// includes operations between the latest refresh() and the one
    /// before that, except waiting for vsync.
    pub fn avg_frame_duration(&self) -> Duration {
        if self.frame_durations.is_empty() {
            Duration::from_millis(0)
        } else {
            let sum = self
                .frame_durations
                .iter()
                .fold(Duration::from_millis(0), |acc, &duration| acc + duration);
            sum / self.frame_durations.len() as u32
        }
    }

    /// Returns the how many frames were rendered during the last
    /// second. This is updated once per second.
    pub fn frames_last_second(&self) -> u32 {
        self.fps
    }
}
