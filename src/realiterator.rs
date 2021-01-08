/// produces: [ linear_interpol(start, end, i/steps) | i <- 0..steps ]
/// (does NOT include "end")
///
/// linear_interpol(a, b, p) = (1 - p) * a + p * b
#[derive(Clone, Debug)]
pub struct FloatIterator {
    current: u64,
    current_back: u64,
    steps: u64,
    start: f64,
    end: f64,
}

impl FloatIterator {
    pub fn new(start: f64, end: f64, steps: u64) -> Self {
        FloatIterator {
            current: 0,
            current_back: steps,
            steps: steps,
            start: start,
            end: end,
        }
    }

    /// calculates number of steps from (end - start) / step
    pub fn new_with_step(start: f64, end: f64, step: f64) -> Self {
        let steps = ((end - start) / step).abs().round() as u64;
        Self::new(start, end, steps)
    }

    pub fn length(&self) -> u64 {
        self.current_back - self.current
    }

    fn at(&self, pos: u64) -> f64 {
        let f_pos = pos as f64 / self.steps as f64;
        (1. - f_pos) * self.start + f_pos * self.end
    }

    /// panics (in debug) when len doesn't fit in usize
    fn usize_len(&self) -> usize {
        let l = self.length();
        debug_assert!(l <= ::std::usize::MAX as u64);
        l as usize
    }
}

impl Iterator for FloatIterator {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.current_back {
            return None;
        }
        let result = self.at(self.current);
        self.current += 1;
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.usize_len();
        (l, Some(l))
    }

    fn count(self) -> usize {
        self.usize_len()
    }
}

impl DoubleEndedIterator for FloatIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current >= self.current_back {
            return None;
        }
        self.current_back -= 1;
        let result = self.at(self.current_back);
        Some(result)
    }
}

impl ExactSizeIterator for FloatIterator {
    fn len(&self) -> usize {
        self.usize_len()
    }

    //fn is_empty(&self) -> bool {
    //    self.length() == 0u64
    //}
}
