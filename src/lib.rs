//! Wrap an iterator, and get progress data as it's executed. A more advanced
//! [`.enumerate()`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.enumerate)
//!
//! # Usage
//! Call `.progress()` on any Iterator, and get a new iterator that yields `(ProgressRecord, T)`, where `T`
//! is the original value. A `ProgressRecord` has many helpful methods to query the current state
//! of the iterator
//!
//! # Example
//!
//! ```
//! use iter_progress::ProgressableIter;
//! // Create an iterator that goes from 0 to 1,000
//! let my_iter = 0..1_000;
//! let mut progressor = my_iter.progress();
//!
//! // This new iterator returns a struct with the current state, and the inner object returned by
//! // the iterator
//! let (state, number) = progressor.next().unwrap();
//! assert_eq!(number, 0);
//! 
//! // We can now use methods on `state` to find out about this object
//!
//! // 0 to 1
//! assert_eq!(state.fraction(), Some(0.001));
//! // We are 0.1% the way through
//! assert_eq!(state.percent(), Some(0.1));
//! ```
//! 
//! There are numerous ergnomic methods for access data on the state of the iterator
//! 
use std::iter::Iterator;
use std::time::{Instant, Duration};

/// Every step of the underlying iterator, one of these is generated. It contains all the
/// information of how this iterator is progresing. Use the methods to access data on it.
///
pub struct ProgressRecord {

    /// How many elements before this
    num: usize,

    /// How long since we started iterating.
    iterating_for: Duration,

    size_hint: (usize, Option<usize>),

    recent_rate: f32,

    /// The timestamp of when the previous record was created. Will be None if this is first.
    previous_record_tm: Option<Instant>,

    /// When the iteration started
    started_iterating: Instant,

}

impl ProgressRecord {

    /// Duration since iteration started
    pub fn duration_since_start(&self) -> Duration {
        self.iterating_for
    }

    /// Number of items we've generated so far. Will be 0 for the first element
    ///
    /// ``
    /// # use iter_progress::ProgressableIter;
    /// let mut progressor = (0..1_000).progress();
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(state.num_done(), 0);
    /// ```
    ///
    /// ```
    /// # use iter_progress::ProgressableIter;
    /// let mut progressor = (0..1_000).progress().skip(120);
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(state.num_done(), 121);
    /// ```
    pub fn num_done(&self) -> usize {
        self.num
    }

    /// `Instant` for when the previous record was generated. None if there was no previous record.
    /// 
    /// This can be useful for calculating fine grained rates
    pub fn previous_record_tm(&self) -> Option<Instant> {
        self.previous_record_tm
    }

    /// When the iteration started
    pub fn started_iterating(&self) -> Instant {
        self.started_iterating
    }

    /// Prints a basic message
    pub fn print_message(&self) {
        println!("{}", self.message());
    }

    /// Returns a basic log message of where we are now. You can construct this yourself, but this
    /// is a helpful convience method.
    /// Currently looks likt "{time_since_start} - Seen {num_see} Rate {rate}/sec", but the library
    /// might change it later. Construct your own message.
    pub fn message(&self) -> String {
        format!("{} - Seen {} Rate {}/sec", self.duration_since_start().as_secs(), self.num_done(), self.recent_rate())
    }

    /// Number of items per second, calcualted from the start
    pub fn rate(&self) -> f32 {
        // number of items per second
        let duration_since_start = self.duration_since_start();
        (self.num_done() as f32) / (duration_since_start.as_secs() as f32)
    }

    /// How far through the iterator as a fraction, if known
    ///
    /// ```
    /// use iter_progress::ProgressableIter;
    /// let mut progressor = (0..1_000).progress().skip(120);
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(num, 120);
    /// assert_eq!(state.fraction(), Some(0.121));
    /// ```
    ///
    /// Returns None if we cannot know, e.g. for an infinite iterator
    /// ```
    /// # use iter_progress::ProgressableIter;
    /// let mut progressor = (0..).progress().skip(120);
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(state.fraction(), None);
    /// ```
    pub fn fraction(&self) -> Option<f32> {
        if self.is_size_known() {
            let remaining = self.size_hint.0;
            let done = self.num_done();
            Some(( done as f32 ) / ( (remaining+done) as f32 ))
        } else {
            None
        }
    }

    /// Percentage progress through the iterator, if known.
    ///
    /// ```
    /// use iter_progress::ProgressableIter;
    /// let mut progressor = (0..1_000).progress().skip(120);
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(state.percent(), Some(12.1));
    /// ```
    ///
    /// Returns None if we cannot know, e.g. for an infinite iterator
    /// ```
    /// # use iter_progress::ProgressableIter;
    /// let mut progressor = (0..).progress().skip(120);
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(state.percent(), None);
    /// ```
    pub fn percent(&self) -> Option<f32> {
        match self.fraction() {
            None => { None }
            Some(f) => { Some(f * 100.0) }
        }
    }

    /// Print out `msg`, but only if there has been `n` seconds since last printout
    pub fn print_every_n_sec<T: std::fmt::Display>(&self, n: f32, msg: T) {
        if self.should_do_every_n_sec(n) {
            print!("{}", msg);
        }
    }

    /// Do thing but only every n sec (as far as possible).
    /// Could be a print statement.
    pub fn do_every_n_sec<F: Fn(&Self)>(&self, n: f32, f: F) {
        if self.should_do_every_n_sec(n) {
            f(self);
        }
    }

    /// If we want to print every `n` sec, should we print now?
    pub fn should_do_every_n_sec(&self, n: f32) -> bool {
        // get the secs since start as a f32
        let duration_since_start = self.duration_since_start();
        let secs_since_start: f32 = duration_since_start.as_secs() as f32 + duration_since_start.subsec_nanos() as f32 / 1_000_000_000.0;

        match self.previous_record_tm() {
            None => {
                // This iteration is the first time, so we should print if more than `n` seconds
                // have gone past
                secs_since_start > n
            },
            Some(last_time) => {
                let last_time_offset = last_time - self.started_iterating();
                let last_time_offset: f32 = last_time_offset.as_secs() as f32 + last_time_offset.subsec_nanos() as f32 / 1_000_000_000.0;

                let current_step = secs_since_start / n;
                let last_step = last_time_offset / n;

                current_step.trunc() > last_step.trunc()
            },
        }
    }

    /// If we want to print every `n` items, should we print now?
    pub fn should_do_every_n_items(&self, n: usize) -> bool {
        (self.num_done() - 1) % n == 0
    }


    /// Print out `msg`, but only if there has been `n` items.
    /// Often you want to print out a debug message every 1,000 items or so. This function does
    /// that.
    pub fn print_every_n_items<T: std::fmt::Display>(&self, n: usize, msg: T) {
        if self.should_do_every_n_items(n) {
            print!("{}", msg);
        }
    }

    /// Do thing but only every `n` items.
    /// Could be a print statement.
    ///
    /// takes 2 arguments, `n` and the function (`f`) which takes a `&ProgressState`. `f` will only
    /// be called every `n` items that pass through the iterator.
    ///
    /// ```
    /// # use iter_progress::ProgressableIter;
    /// for (state, _) in (0..150).progress() {
    ///    state.do_every_n_items(5, |state| {
    ///        println!("Current progress: {}%", state.percent().unwrap());
    ///    });
    /// }
    /// ```
    pub fn do_every_n_items<F: Fn(&Self)>(&self, n: usize, f: F) {
        if self.should_do_every_n_items(n) {
            f(self);
        }
    }

    /// Do we know how big this iterator is?
    /// False iff there is some ambiguity/unknown
    fn is_size_known(&self) -> bool {
        match self.size_hint.1 {
            None => { false },
            Some(x) => { self.size_hint.0 == x },
        }
    }

    /// The rate of the last few items.
    pub fn recent_rate(&self) -> f32 {
        self.recent_rate
    }

}

/// Wraps an iterator and keeps track of state used for `ProgressRecord`'s
pub struct ProgressRecorderIter<I> {

    /// The iterator that we are iteating on
    iter: I,

    /// How many items have been seen
    count: usize,

    /// When did we start iterating
    started_iterating: Instant,

    /// Keeps track of recent times
    recent_times: Vec<Instant>
}

impl<I: Iterator> ProgressRecorderIter<I> {
    /// Create a new `ProgressRecorderIter` from another iterator.
    pub fn new(iter: I) -> ProgressRecorderIter<I> {
        ProgressRecorderIter{ iter: iter, count: 0, started_iterating: Instant::now(), recent_times: Vec::with_capacity(5) }
    }

    /// Calculate the current `ProgressRecord` for where we are now.
    fn generate_record(&mut self) -> ProgressRecord {
        // recent_times is a vec of times, with newer times at the end. However it'll always be <
        // 100 elements long.
        let now = Instant::now();
        self.recent_times.push(now);
        while self.recent_times.len() > 100 {
            self.recent_times.remove(0);
        }

        let recent_rate = match self.recent_times.get(0) {
            None => ::std::f32::INFINITY,
            Some(&first) => {
                let dur = now - first;
                (self.recent_times.len() as f32 ) / (dur.as_secs() as f32)
            },
        };

        // last element of recent_times will be the current time, for this record. so second last
        // will be the previous time. In python we'd do [-1] for the last, and [-2] for second
        // last.
        let previous_record_tm = match self.recent_times.len() {
            0 | 1 => { None },
            _ => {
                self.recent_times.get(self.recent_times.len()-2).map(|t| { t.clone() })
            }
        };

        self.count += 1;
        ProgressRecord{ num: self.count, iterating_for: now - self.started_iterating, size_hint: self.iter.size_hint(), recent_rate: recent_rate, previous_record_tm: previous_record_tm, started_iterating: self.started_iterating }
    }

    /// Gets the original iterator back
    pub fn into_inner(self) -> I {
        self.iter
    }

}

/// An iterator that records it's progress as it goes along
pub trait ProgressableIter<I> {
    fn progress(self) -> ProgressRecorderIter<I>;
}

impl<I> ProgressableIter<I> for I where I: Iterator {
    /// Convert an iterator into a `ProgressRecorderIter`.
    fn progress(self) -> ProgressRecorderIter<I> {
        ProgressRecorderIter::new(self)
    }
}


impl<I> Iterator for ProgressRecorderIter<I> where I: Iterator {
    type Item = (ProgressRecord, <I as Iterator>::Item);


    #[inline]
    fn next(&mut self) -> Option<(ProgressRecord, <I as Iterator>::Item)> {
        self.iter.next().map(|a| {
            (self.generate_record(), a)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }
}



mod test {
    #[test]
    fn test_simple() {
        use super::ProgressableIter;
        use std::thread::sleep;
        use std::time::Duration;

        let vec: Vec<u8> = vec![0, 1, 2, 3, 4];
        let mut progressor = vec.iter().progress();

        sleep(Duration::from_millis(500));
        let (state, _) = progressor.next().unwrap();
        assert_eq!(state.message(), "0 - Seen 1 Rate inf/sec");
        // It'll always print on the first one
        assert_eq!(state.should_do_every_n_items(2), true);
        assert_eq!(state.should_do_every_n_items(3), true);
        assert_eq!(state.should_do_every_n_items(5), true);
        assert_eq!(state.rate(), ::std::f32::INFINITY);
        assert_eq!(state.recent_rate(), ::std::f32::INFINITY);
        // First run, so there should be nothing here
        assert!(state.previous_record_tm().is_none());

        assert_eq!(state.should_do_every_n_sec(1.), false);
        assert_eq!(state.should_do_every_n_sec(2.), false);
        assert_eq!(state.should_do_every_n_sec(0.3), true);


        sleep(Duration::from_millis(500));

        let (state, _) = progressor.next().unwrap();
        assert_eq!(state.message(), "1 - Seen 2 Rate inf/sec");
        assert_eq!(state.should_do_every_n_items(2), false);
        assert_eq!(state.should_do_every_n_items(3), false);
        assert_eq!(state.should_do_every_n_items(5), false);
        assert_eq!(state.rate(), 2.);
        //assert_eq!(state.recent_rate(), 2.);
        // This'll be the time for the first one
        assert!(state.previous_record_tm().is_some());
        let since_last_time = state.previous_record_tm().unwrap().elapsed();
        assert!(since_last_time < Duration::from_millis(550));
        assert!(since_last_time >= Duration::from_millis(500));
        assert_eq!(state.should_do_every_n_sec(1.), true);
        assert_eq!(state.should_do_every_n_sec(2.), false);
        assert_eq!(state.should_do_every_n_sec(0.8), true);

        sleep(Duration::from_millis(500));
        let (state, _) = progressor.next().unwrap();
        assert_eq!(state.message(), "1 - Seen 3 Rate 3/sec");
        assert_eq!(state.should_do_every_n_items(2), true);
        assert_eq!(state.should_do_every_n_items(3), false);
        assert_eq!(state.should_do_every_n_items(5), false);
        assert_eq!(state.rate(), 3.);
        assert_eq!(state.recent_rate(), 3.);
        assert_eq!(state.should_do_every_n_sec(1.), false);
        assert_eq!(state.should_do_every_n_sec(2.), false);
        assert_eq!(state.should_do_every_n_sec(0.8), false);
        assert_eq!(state.should_do_every_n_sec(1.5), true);

        sleep(Duration::from_millis(500));
        let (state, _) = progressor.next().unwrap();
        assert_eq!(state.message(), "2 - Seen 4 Rate 4/sec");
        assert_eq!(state.should_do_every_n_items(2), false);
        assert_eq!(state.should_do_every_n_items(3), true);
        assert_eq!(state.should_do_every_n_items(5), false);
        assert_eq!(state.rate(), 2.);
        assert_eq!(state.recent_rate(), 4.);
    }

    #[test]
    fn test_size_hint() {
        use super::ProgressableIter;

        let vec: Vec<u8> = vec![0, 1, 2, 3, 4];
        let mut progressor = vec.iter().progress();

        let (state, _) = progressor.next().unwrap();
        assert_eq!(state.fraction(), Some(0.2));
        assert_eq!(state.percent(), Some(20.));

        let (state, _) = progressor.next().unwrap();
        assert_eq!(state.fraction(), Some(0.4));
        assert_eq!(state.percent(), Some(40.));

        let mut progressor = (0..).progress();

        let (state, val) = progressor.next().unwrap();
        assert_eq!(val, 0);
        assert_eq!(state.fraction(), None);
        let (state, val) = progressor.next().unwrap();
        assert_eq!(val, 1);
        assert_eq!(state.fraction(), None);

    }
}

