/// Fixed-size circular buffer for time-series graph data.
/// When full, oldest entries are overwritten.
pub struct RingBuffer<T> {
    data: Vec<T>,
    capacity: usize,
    head: usize,
    len: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![T::default(); capacity.max(1)],
            capacity: capacity.max(1),
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % self.capacity;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    pub fn iter_ordered(&self) -> impl Iterator<Item = &T> {
        let start = if self.len < self.capacity {
            0
        } else {
            self.head
        };
        (0..self.len).map(move |i| &self.data[(start + i) % self.capacity])
    }

    pub fn latest(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            let idx = (self.head + self.capacity - 1) % self.capacity;
            Some(&self.data[idx])
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn resize(&mut self, new_capacity: usize) {
        let collected: Vec<T> = self.iter_ordered().cloned().collect();
        let cap = new_capacity.max(1);
        self.data = vec![T::default(); cap];
        self.capacity = cap;
        self.head = 0;
        self.len = 0;
        let skip = collected.len().saturating_sub(cap);
        for v in collected.into_iter().skip(skip) {
            self.push(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let rb: RingBuffer<f32> = RingBuffer::new(5);
        assert_eq!(rb.len(), 0);
        assert!(rb.latest().is_none());
    }

    #[test]
    fn push_partial() {
        let mut rb: RingBuffer<f32> = RingBuffer::new(5);
        rb.push(1.0);
        rb.push(2.0);
        rb.push(3.0);
        let v: Vec<f32> = rb.iter_ordered().copied().collect();
        assert_eq!(v, vec![1.0, 2.0, 3.0]);
        assert_eq!(rb.latest(), Some(&3.0));
    }

    #[test]
    fn push_wraps() {
        let mut rb: RingBuffer<f32> = RingBuffer::new(3);
        for i in 1..=5 {
            rb.push(i as f32);
        }
        let v: Vec<f32> = rb.iter_ordered().copied().collect();
        assert_eq!(v, vec![3.0, 4.0, 5.0]);
        assert_eq!(rb.latest(), Some(&5.0));
        assert_eq!(rb.len(), 3);
    }

    #[test]
    fn resize_keeps_recent() {
        let mut rb: RingBuffer<f32> = RingBuffer::new(5);
        for i in 1..=5 {
            rb.push(i as f32);
        }
        rb.resize(3);
        let v: Vec<f32> = rb.iter_ordered().copied().collect();
        assert_eq!(v, vec![3.0, 4.0, 5.0]);
    }
}
