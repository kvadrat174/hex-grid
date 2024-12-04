use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub struct TempNode {
    pub x: usize,
    pub y: usize,
    pub passable: bool,
    pub passability: f64,
    pub g: f64,
    pub h: Option<f64>,
    pub f: f64,
    pub opened: Option<bool>,
    pub closed: Option<bool>,
    pub parent: Option<(usize, usize)>,
}

impl TempNode {
    pub fn new(x: usize, y: usize, passable: bool, passability: f64) -> Self {
        Self {
            x,
            y,
            passable,
            passability,
            g: -1.0,
            h: None,
            f: -1.0,
            opened: None,
            closed: None,
            parent: None,
        }
    }
    pub fn calculate_f(&mut self) {
        self.f = self.g + self.h.unwrap_or(0.0);
    }

    pub fn set_closed(&mut self, v: bool) {
        self.closed = Some(v)
    }

    pub fn set_opened(&mut self, v: bool) {
        self.opened = Some(v)
    }

    pub fn set_g(&mut self, v: f64) {
        self.g = v
    }

    pub fn set_h(&mut self, v: f64) {
        self.h = Some(v)
    }

    pub fn set_parent(&mut self, v: (usize, usize)) {
        self.parent = Some(v)
    }

    pub fn set_passability(&mut self, v: f64) {
        self.passability = v;
    }

    pub fn set_passable(&mut self, v: bool) {
        self.passable = v;
    }

    pub fn reset(&mut self) {
        self.g = -1.0;
        self.h = None;
        self.f = -1.0;
        self.opened = None;
        self.closed = None;
        self.parent = None;
    }
}

impl PartialEq for TempNode {
    fn eq(&self, other: &Self) -> bool {
        // Compare x, y, passable, passability, and f only if `f` is not NaN
        self.x == other.x
            && self.y == other.y
            && self.passable == other.passable
            && self.passability == other.passability
            && (self.f.is_nan() == other.f.is_nan() || self.f == other.f) // f64 check
    }
}

impl Eq for TempNode {}

impl Ord for TempNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.f.partial_cmp(&other.f).unwrap_or(Ordering::Equal) // Reverse order for a min-heap
    }
}

impl PartialOrd for TempNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
