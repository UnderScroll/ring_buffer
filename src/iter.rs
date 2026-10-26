use crate::RingBuffer;

impl<const CAPACITY: usize, T> IntoIterator for RingBuffer<CAPACITY, T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        let (head, tail) = self.slices();
        let mut flattened_buffer = Vec::with_capacity(self.len);

        // SAFETY: `head` and `tail` hold exactly `len` initialized items, and the destination has capacity for `len`. No items overlap.
        unsafe {
            let dst = flattened_buffer.as_mut_ptr();
            std::ptr::copy_nonoverlapping(head.as_ptr(), dst, head.len());
            std::ptr::copy_nonoverlapping(tail.as_ptr(), dst.add(head.len()), tail.len());
            flattened_buffer.set_len(self.len);
            self.len = 0; // SAFETY: Makes sure items aren't droped by the RingBuffer drop
        }

        flattened_buffer.into_iter()
    }
}
