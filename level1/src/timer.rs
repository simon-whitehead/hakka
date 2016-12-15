pub struct FrameTimer {
    pub elapsed: f64,
    pub fps: u32,
    pub interval: u32,
    pub prev: u32,
    pub last_second: u32,

    pub last_fps: u32, // The last captured FPS we had
    pub ticks: u32,
}

impl FrameTimer {
    pub fn new(interval: u32, previous: u32, last_second: u32, fps: u32) -> FrameTimer {

        FrameTimer {
            interval: interval,
            prev: previous,
            last_second: last_second,
            fps: fps,
            elapsed: 0f64,

            last_fps: 0,
            ticks: 0,
        }
    }
}