use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
struct HeapEntry {
    cost: f64,
    x: usize,
    y: usize,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for HeapEntry {}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Use f64's total_cmp for more efficient comparison without unwrap
        other.cost.total_cmp(&self.cost)
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct CustomHeap {
    heap: BinaryHeap<HeapEntry>,
}

impl CustomHeap {
    pub fn new(capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, value: (f64, usize, usize)) {
        let entry = HeapEntry {
            cost: value.0,
            x: value.1,
            y: value.2,
        };
        self.heap.push(entry);
    }

    pub fn pop(&mut self) -> Option<(usize, usize)> {
        self.heap.pop().map(|entry| (entry.x, entry.y))
    }

    pub fn update(&mut self, value: (f64, usize, usize)) {
        // Create a temporary entry for comparison
        let temp_entry = HeapEntry {
            cost: value.0,
            x: value.1,
            y: value.2,
        };

        // More efficient update using retain
        self.heap.retain(|e| e.x != temp_entry.x || e.y != temp_entry.y);
        self.heap.push(temp_entry);
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}