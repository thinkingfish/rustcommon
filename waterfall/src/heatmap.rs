use clocksource::precise::{Duration, Instant, UnixInstant};
use histogram::Histogram;

/// A time-series collection of histograms, where each slice represents a fixed
/// time interval.
pub struct Heatmap {
    slices: Vec<Histogram>,
    config: histogram::Config,
    resolution: Duration,
    start: UnixInstant,
    current_tick: usize,
}

impl Heatmap {
    /// Create a new `Heatmap`.
    ///
    /// * `grouping_power` - controls the bucket granularity (see [`histogram::Config`])
    /// * `max_value_power` - controls the max representable value
    /// * `span` - total time window the heatmap covers
    /// * `resolution` - duration of each time slice
    pub fn new(
        grouping_power: u8,
        max_value_power: u8,
        span: Duration,
        resolution: Duration,
    ) -> Result<Self, histogram::Error> {
        let config = histogram::Config::new(grouping_power, max_value_power)?;
        let num_slices = (span.as_nanos() / resolution.as_nanos()) as usize;
        let slices = (0..num_slices)
            .map(|_| Histogram::with_config(&config))
            .collect();

        Ok(Self {
            slices,
            config,
            resolution,
            start: UnixInstant::now(),
            current_tick: 0,
        })
    }

    /// Record a value at the given instant.
    pub fn increment(
        &mut self,
        now: Instant,
        value: u64,
        count: u64,
    ) -> Result<(), histogram::Error> {
        let elapsed = now.elapsed();
        // This is a rough approach: use the monotonic clock offset to pick a slice
        let tick = if self.resolution.as_nanos() > 0 {
            (elapsed.as_nanos() / self.resolution.as_nanos()) as usize
        } else {
            0
        };
        // Wrap around if we exceed the number of slices
        let idx = tick % self.slices.len();

        // If we've advanced past our previous tick, clear stale slices
        if tick > self.current_tick {
            let start_clear = (self.current_tick + 1) % self.slices.len();
            let end_clear = (tick + 1) % self.slices.len();
            if tick - self.current_tick >= self.slices.len() {
                // Clear all slices
                for s in &mut self.slices {
                    for v in s.as_mut_slice().iter_mut() {
                        *v = 0;
                    }
                }
            } else if end_clear > start_clear {
                for i in start_clear..end_clear {
                    for v in self.slices[i].as_mut_slice().iter_mut() {
                        *v = 0;
                    }
                }
            } else {
                for i in start_clear..self.slices.len() {
                    for v in self.slices[i].as_mut_slice().iter_mut() {
                        *v = 0;
                    }
                }
                for i in 0..end_clear {
                    for v in self.slices[i].as_mut_slice().iter_mut() {
                        *v = 0;
                    }
                }
            }
            self.current_tick = tick;
        }

        self.slices[idx].add(value, count)
    }

    /// Returns the number of active (non-empty or allocated) slices.
    pub fn active_slices(&self) -> usize {
        self.slices.len()
    }

    /// Returns the number of buckets per histogram slice.
    pub fn buckets(&self) -> usize {
        self.config.total_buckets()
    }

    /// Returns the time resolution (duration of each slice).
    pub fn resolution(&self) -> Duration {
        self.resolution
    }

    /// Returns the start time of the heatmap.
    pub fn start_at(&self) -> UnixInstant {
        self.start
    }
}

impl<'a> IntoIterator for &'a Heatmap {
    type Item = &'a Histogram;
    type IntoIter = std::slice::Iter<'a, Histogram>;

    fn into_iter(self) -> Self::IntoIter {
        self.slices.iter()
    }
}
