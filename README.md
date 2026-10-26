# Ring Buffer

Usage:
```rs
use ring_buffer::RingBuffer;

fn main() {
  // Create `RingBuffer`
  let mut ring_buffer = RingBuffer::<16, _>::new();

  // Push items
  for i in 0..20 {
    ring_buffer.push_overwrite(i);
  }

  // Access items (always in reverse order)
  for item in ring_buffer.reverse_iter() {
    println!("{item}");
  }

  // Display
  println!("{ring_buffer:#?}");
}
```
