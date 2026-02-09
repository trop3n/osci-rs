# 03 - Ownership & Borrowing

## Overview

This document explains Rust's ownership system - the key feature that enables memory safety without garbage collection.

## The Three Rules

Rust's ownership system follows three simple rules:

1. **Each value has exactly one owner**
2. **When the owner goes out of scope, the value is dropped**
3. **You can have either one mutable reference OR many immutable references, but not both**

---

## Ownership in Practice

### Move Semantics

When you assign a value to another variable, ownership **moves**:

```rust
let buffer = SampleBuffer::new(1024);
let buffer2 = buffer;  // Ownership moves to buffer2

// buffer.get_samples();  // ERROR! buffer is no longer valid
buffer2.get_samples();    // OK - buffer2 owns it now
```

### Clone vs Move

Some types implement `Clone`, allowing explicit copies:

```rust
let s1 = String::from("hello");
let s2 = s1.clone();  // Explicit copy

println!("{} {}", s1, s2);  // Both valid!
```

### Copy Types

Simple types that live on the stack implement `Copy` and are copied automatically:

```rust
let x = 42;
let y = x;  // x is copied, not moved

println!("{} {}", x, y);  // Both valid!
```

`Copy` types include: integers, floats, booleans, chars, tuples of Copy types.

---

## Borrowing

Instead of transferring ownership, you can **borrow** a value using references.

### Immutable References (`&T`)

```rust
fn print_samples(buffer: &SampleBuffer) {
    let samples = buffer.get_samples();
    println!("Got {} samples", samples.len());
}

let buffer = SampleBuffer::new(1024);
print_samples(&buffer);  // Borrow immutably
print_samples(&buffer);  // Can borrow again!
// buffer still valid here
```

### Mutable References (`&mut T`)

```rust
fn add_sample(buffer: &mut SampleBuffer) {
    buffer.push(XYSample::new(0.5, 0.5));
}

let mut buffer = SampleBuffer::new(1024);
add_sample(&mut buffer);  // Borrow mutably
```

### The Borrowing Rules

```rust
let mut data = vec![1, 2, 3];

// OK: Multiple immutable borrows
let a = &data;
let b = &data;
println!("{:?} {:?}", a, b);

// OK: One mutable borrow
let c = &mut data;
c.push(4);

// ERROR: Can't have both mutable and immutable
let d = &data;
let e = &mut data;  // ERROR!
```

---

## Real Example: The Borrow Checker Error

We encountered this error in our buffer code:

```rust
// ERROR: Cannot borrow `inner` as immutable and mutable
inner.samples[inner.write_pos] = sample;
```

**Why it fails:**
- `inner.samples[...]` borrows `inner` mutably (to modify)
- `inner.write_pos` borrows `inner` immutably (to read index)
- Both happen in the same expression!

**The fix:**

```rust
// Copy the index first
let pos = inner.write_pos;  // Copy the value
inner.samples[pos] = sample;  // Now only one borrow
```

---

## Shared Ownership with `Arc`

Sometimes multiple parts of your program need to own the same data. `Arc` (Atomic Reference Counted) enables this:

```rust
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);

let data_clone = Arc::clone(&data);  // New reference, same data

// Both data and data_clone point to the same Vec
// Data is freed when the last Arc is dropped
```

### Arc in Our Code

```rust
// In SampleBuffer
inner: Arc<Mutex<BufferInner>>,

// Create a clone for the audio thread
pub fn clone_ref(&self) -> Self {
    Self {
        inner: Arc::clone(&self.inner),  // Share the same data
    }
}
```

---

## Interior Mutability with `Mutex`

`Arc` gives shared ownership, but Rust still prevents multiple mutable accesses. `Mutex` provides **interior mutability** - the ability to mutate through a shared reference:

```rust
use std::sync::Mutex;

let data = Mutex::new(vec![1, 2, 3]);

// Lock to get mutable access
{
    let mut guard = data.lock().unwrap();
    guard.push(4);  // Mutate through the MutexGuard
}  // Lock released when guard goes out of scope

// Lock again elsewhere
{
    let guard = data.lock().unwrap();
    println!("{:?}", *guard);
}
```

### try_lock for Non-blocking Access

```rust
// For real-time code (audio), we can't wait for a lock
if let Ok(mut guard) = data.try_lock() {
    // Got the lock - do work
    guard.push(5);
} else {
    // Lock was held - skip this sample
    // This is OK for visualization
}
```

---

## Arc<Mutex<T>> Pattern

Combining `Arc` and `Mutex` is the standard pattern for shared mutable state across threads:

```rust
// Create shared buffer
let buffer = Arc::new(Mutex::new(Vec::new()));

// Clone for another thread
let buffer_clone = Arc::clone(&buffer);

// Thread 1: Producer
std::thread::spawn(move || {
    let mut guard = buffer_clone.lock().unwrap();
    guard.push(1);
});

// Main thread: Consumer
let guard = buffer.lock().unwrap();
println!("{:?}", *guard);
```

---

## Key Takeaways

1. **Ownership prevents use-after-free** - The compiler tracks who owns what
2. **Borrowing prevents data races** - Only one mutable reference at a time
3. **Arc enables shared ownership** - Multiple owners, reference counted
4. **Mutex enables interior mutability** - Safe mutation through shared references
5. **try_lock for real-time** - Non-blocking access for audio threads

---

## Common Patterns

### Taking Ownership in Functions

```rust
fn consume(buffer: SampleBuffer) {
    // buffer is dropped at end of function
}
```

### Borrowing for Reading

```rust
fn read(buffer: &SampleBuffer) -> Vec<XYSample> {
    buffer.get_samples()
}
```

### Borrowing for Mutation

```rust
fn modify(buffer: &mut SampleBuffer) {
    buffer.push(XYSample::default());
}
```

### Returning Owned Data

```rust
fn create() -> SampleBuffer {
    SampleBuffer::new(1024)  // Ownership transfers to caller
}
```

---

## Exercises

1. Try removing the `let pos = inner.write_pos;` fix and see the error
2. Create a function that takes `&SampleBuffer` and prints sample count
3. Try creating two `&mut` references to the same data - observe the error
4. Use `Arc::strong_count()` to track how many references exist

---

## Links

- [Rust Book: Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Rust Book: References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [Rust Book: Shared State](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
