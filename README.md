# iter-progress

[![Build Status](https://travis-ci.org/rory/iter-progress-rs.svg?branch=master)](https://travis-ci.org/rory/iter-progress-rs)
[![Crates.io](https://img.shields.io/crates/v/iter-progress.svg)](https://crates.io/crates/iter-progress)
[![Documentation](https://docs.rs/iter-progress/badge.svg)](https://docs.rs/iter-progress/)

Wrap an iterator, and get progress data as it's executed. A more advanced
[`.enumerate()`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.enumerate)

# [Documentation](https://docs.rs/iter-progress/)

Wrap an iterator, and get progress data as it's executed. A more advanced
[`.enumerate()`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.enumerate)

# Usage
Call `.progress()` on any Iterator, and get a new iterator that yields `(ProgressRecord, T)`, where `T`
is the original value. A `ProgressRecord` has many helpful methods to query the current state
of the iterator

# Example

```rust
use iter_progress::ProgressableIter;
// Create an iterator that goes from 0 to 1,000
let my_iter = 0..1_000;
let mut progressor = my_iter.progress();

// This new iterator returns a struct with the current state, and the inner object returned by
// the iterator
let (state, number) = progressor.next().unwrap();
assert_eq!(number, 0);

// We can now use methods on `state` to find out about this object

// If we know the size of the iterator, we can query how far we are through it
// How far through the iterator are we. 0 to 1
assert_eq!(state.fraction(), Some(0.001));

// We are 0.1% the way through
assert_eq!(state.percent(), Some(0.1));
```

Another usage:

```rust
use iter_progress::ProgressableIter;
let my_big_vec = vec![false; 100];

for (state, val) in my_big_vec.iter().progress() {
    // Every 1 second, execute this function with the the `state`
    state.do_every_n_sec(1., |state| {
       println!("{}% the way though, and doing {} per sec.", state.percent().unwrap(), state.rate());
    });

    // Do something to process `val`
}
```

`.do_every_n_sec` is a "best effort" attempt. It's single threaded, so will be called if the
last time that was called was more than N sec ago. `.do_every_n_items` is called every N items.

