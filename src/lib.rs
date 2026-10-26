use std::{
    fmt::Debug,
    iter::Chain,
    mem::{MaybeUninit, needs_drop},
    slice,
};

pub mod iter;

/// List with limited capacity where the newest item overwrites the oldest when full
pub struct RingBuffer<const CAPACITY: usize, T> {
    buffer: [MaybeUninit<T>; CAPACITY],
    len: usize,
    cursor: usize,
}

impl<const CAPACITY: usize, T> RingBuffer<CAPACITY, T> {
    #[must_use]
    pub const fn new() -> Self {
        const { assert!(CAPACITY >= 1, "RingBuffer's CAPACITY must be at least 1") };

        Self {
            buffer: [const { MaybeUninit::uninit() }; CAPACITY],
            cursor: 0,
            len: 0,
        }
    }

    /// Push an item at the start of the buffer, overwriting the oldest item if the buffer is full
    pub fn push_overwrite(&mut self, item: T) {
        // Overwrite with item at write index
        self.buffer[self.cursor] = MaybeUninit::new(item);

        // Update len
        self.increase_len();

        // Update cursor
        self.advance_cursor();
    }

    /// Returns an iterator over the RingBuffer's items in reverse order (last to first)
    pub fn reverse_iter(&self) -> Chain<slice::Iter<'_, T>, slice::Iter<'_, T>> {
        let (a, b) = self.slices();
        a.iter().chain(b.iter())
    }

    /// Returns an iterator that allows modyfing the RingBuffer's items in reverse order (last to first)
    pub fn reverse_iter_mut(&mut self) -> Chain<slice::IterMut<'_, T>, slice::IterMut<'_, T>> {
        let (a, b) = self.slices_mut();
        a.iter_mut().chain(b.iter_mut())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn advance_cursor(&mut self) {
        self.cursor = self.cursor.wrapping_add(1);
        if self.cursor == CAPACITY {
            self.cursor = 0
        }
    }

    fn increase_len(&mut self) {
        self.len += (self.len < CAPACITY) as usize; // a `bool` is guarenteed to be 0/1 as usize
    }

    fn is_full(&self) -> bool {
        self.len == CAPACITY
    }

    fn slices(&self) -> (&[T], &[T]) {
        if self.is_full() {
            // SAFETY: We know self.write_cursor is a valid index in the buffer
            let (tail, head) = unsafe { self.buffer.split_at_unchecked(self.cursor) };

            // SAFETY: We know the ring buffer is full, therefor all elements are initialized
            unsafe { (assume_slice_init(head), assume_slice_init(tail)) }
        } else {
            let slice = &self.buffer[..self.len];

            // SAFETY: We know all items in range 0..self.len are initialized
            unsafe { (assume_slice_init(slice), &[]) }
        }
    }

    fn slices_mut(&mut self) -> (&mut [T], &mut [T]) {
        if self.is_full() {
            // SAFETY: We know self.write_cursor is a valid index in the buffer
            let (tail, head) = unsafe { self.buffer.split_at_mut_unchecked(self.cursor) };

            // SAFETY: We know the ring buffer is full, therefor all elements are initialized
            unsafe { (assume_slice_mut_init(tail), assume_slice_mut_init(head)) }
        } else {
            let slice = &mut self.buffer[..self.len];

            // SAFETY: We know all items in range 0..self.len are initialized
            unsafe { (assume_slice_mut_init(slice), &mut []) }
        }
    }
}

impl<const CAPACITY: usize, T> Default for RingBuffer<CAPACITY, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAPACITY: usize, T> Drop for RingBuffer<CAPACITY, T> {
    fn drop(&mut self) {
        if needs_drop::<T>() {
            if self.is_full() {
                // SAFETY: We know the ring buffer is full, therefor all elements are initialized
                unsafe {
                    for item in &mut self.buffer {
                        item.assume_init_drop();
                    }
                }
            } else {
                let slice = &mut self.buffer[..self.len];

                // SAFETY: We know all items in range 0..self.len are initialized
                unsafe {
                    for item in slice {
                        item.assume_init_drop();
                    }
                }
            }
        }
    }
}

enum MaybeInit<'a, T> {
    Init { value: &'a T },
    Uninit,
}

impl<'a, T: Debug> Debug for MaybeInit<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Init { value } => (**value).fmt(f),
            Self::Uninit => write!(f, "Uninit"),
        }
    }
}

impl<const CAPACITY: usize, T: Debug> Debug for RingBuffer<CAPACITY, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buffer: Vec<MaybeInit<T>> = if self.is_full() {
            // SAFETY: We know the ring buffer is full, therefor all elements are initialized
            let slice = unsafe { assume_slice_init(&self.buffer) };

            slice
                .iter()
                .map(|item| MaybeInit::Init { value: item })
                .collect()
        } else {
            // SAFETY: We know self.write_cursor() is a valid index in the buffer
            let (tail, head) = unsafe { self.buffer.split_at_unchecked(self.cursor) };

            // SAFETY: We know all items until `write_cursor` are init
            let tail = unsafe { assume_slice_init(tail) }
                .iter()
                .map(|item| MaybeInit::Init { value: item });

            let head = head.iter().map(|_| MaybeInit::Uninit);

            tail.chain(head).collect()
        };

        f.debug_struct("ReverseRingBuffer")
            .field("buffer", &buffer)
            .field("len", &self.len)
            .field("cursor", &self.cursor)
            .finish()
    }
}

// SAFETY: All items in slice must be init
unsafe fn assume_slice_init<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    unsafe { (slice as *const [MaybeUninit<T>] as *const [T]).as_ref_unchecked() }
}

// SAFETY: All items in slice must be init
unsafe fn assume_slice_mut_init<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    unsafe { (slice as *const [MaybeUninit<T>] as *mut [T]).as_mut_unchecked() }
}
