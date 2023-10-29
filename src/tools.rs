use std::fmt;

/// Mini struct to create render statistics easily
pub struct RenderStats {
    begin: std::time::Instant,
    pub average: std::time::Duration,
    pub max: std::time::Duration,
    pub min: std::time::Duration,
    pub total: std::time::Duration,
    pub frames: u32,
}

impl RenderStats {
    pub fn new() -> Self {
        Self {
            begin: std::time::Instant::now(),
            average: Default::default(),
            max: Default::default(),
            min: std::time::Duration::MAX,
            total: Default::default(),
            frames: 0
        }
    }

    pub fn begin_capture(&mut self) {
        self.begin = std::time::Instant::now();
    }

    pub fn end_capture(&mut self) {
        self.frames += 1;
        let time = std::time::Instant::now();
        let passed = time - self.begin;
        if passed > self.max {
            self.max = passed;
        }
        if passed < self.min {
            self.min = passed;
        }
        self.total += passed;
        self.average = self.total / self.frames;
    }

    pub fn reset(&mut self) {
        self.frames = 0;
        self.average = Default::default();
        self.max = Default::default();
        self.min = std::time::Duration::MAX;
        self.total = Default::default();
    }
}

impl fmt::Display for RenderStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(
            format_args!("Frames: {}, avg: {} ms, min: {} ms, max: {} ms",
        self.frames, 
        self.average.as_micros() as f64/1000., 
        self.min.as_micros() as f64/1000., 
        self.max.as_micros() as f64/1000.))?;
        Ok(())
    }
}