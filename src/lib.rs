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
//! // If we know the size of the iterator, we can query how far we are through it
//! // How far through the iterator are we. 0 to 1
//! assert_eq!(state.fraction(), Some(0.001));
//!
//! // We are 0.1% the way through
//! assert_eq!(state.percent(), Some(0.1));
//! ```
//!
//! Another usage:
//!
//! ```
//! use iter_progress::ProgressableIter;
//! # let my_big_vec = vec![false; 100];
//!
//! for (state, val) in my_big_vec.iter().progress() {
//!     // Every 1 second, execute this function with the the `state`
//!     state.do_every_n_sec(1., |state| {
//!        println!("{}% the way though, and doing {} per sec.", state.percent().unwrap(), state.rate());
//!     });
//!
//!     // Do something to process `val`
//! }
//! ```
//!
//! `.do_every_n_sec` is a "best effort" attempt. It's single threaded, so will be called if the
//! last time that was called was more than N sec ago. `.do_every_n_items` is called every N items.

use std::iter::Iterator;
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests;

/// Every step of the underlying iterator, one of these is generated. It contains all the
/// information of how this iterator is progresing. Use the methods to access data on it.
#[derive(Debug)]
pub struct ProgressRecord {
    /// How many elements before this
    num: usize,

    /// How long since we started iterating.
    iterating_for: Duration,

    /// Value of underlying iterator's `.size_hint()`
    size_hint: (usize, Option<usize>),

    /// If `.assumed_size(...)` was set on `ProgressableIter`, return that.
    assumed_size: Option<usize>,

    /// If we have overridden the calculated fraction
    assumed_fraction: Option<f64>,

    /// The timestamp of when the previous record was created. Will be None if this is first.
    previous_record_tm: Option<Instant>,

    /// When the iteration started
    started_iterating: Instant,

    /// The rolling average duration, if calculated
    rolling_average_duration: Option<Duration>,

    /// The exponential average duration, if calculated
    exp_average_duration: Option<Duration>,
}

impl ProgressRecord {
    /// Duration since iteration started
    pub fn duration_since_start(&self) -> Duration {
        self.iterating_for
    }

    /// Number of items we've generated so far. Will be 0 for the first element
    ///
    /// ```rust
    /// # use iter_progress::ProgressableIter;
    /// let mut progressor = (0..1_000).progress();
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(state.num_done(), 1);
    /// ```
    ///
    ///
    /// ```rust
    /// # use iter_progress::ProgressableIter;
    /// let mut progressor = (0..1_000).progress().skip(10);
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(state.num_done(), 11);
    /// ```
    pub fn num_done(&self) -> usize {
        self.num
    }

    /// The `Instant` for when the previous record was generated. None if there was no previous
    /// record.
    ///
    /// This can be useful for calculating fine-grained rates
    pub fn previous_record_tm(&self) -> Option<Instant> {
        self.previous_record_tm
    }

    /// Return the time `Instant` that this iterator started
    pub fn started_iterating(&self) -> Instant {
        self.started_iterating
    }

    /// Number of items per second, calculated from the start
    pub fn rate(&self) -> f64 {
        // number of items per second
        (self.num_done() as f64) / self.duration_since_start().as_secs_f64()
    }

    /// How far through the iterator as a fraction, if known.
    /// First looks at the `assumed_fraction` if you have overridden that.
    /// Uses the underlying iterator's `.size_hint()` method if that is an exact value, falling
    /// back to any assumed size (set with `.assume_size(...)`). Otherwise returns `None`.
    ///
    /// ```
    /// use iter_progress::ProgressableIter;
    /// let mut progressor = (0..1_000).progress().skip(120);
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(num, 120);
    /// assert_eq!(state.fraction(), Some(0.121));
    /// ```
    ///
    /// Returns `None` if we cannot know, e.g. for an infinite iterator
    /// ```
    /// # use iter_progress::ProgressableIter;
    /// let mut progressor = (0..).progress().skip(120);
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(state.fraction(), None);
    /// ```
    pub fn fraction(&self) -> Option<f64> {
        if self.assumed_fraction.is_some() {
            return self.assumed_fraction;
        }

        let total = if self.size_hint.1 == Some(self.size_hint.0) {
            // use that directly
            Some(self.size_hint.0 + self.num_done())
        } else if self.assumed_size.is_some() {
            self.assumed_size
        } else {
            None
        };

        match total {
            None => None,
            Some(total) => {
                let done = self.num_done();
                Some((done as f64) / (total as f64))
            }
        }
    }

    /// Assume that this is actually at this fraction through
    pub fn assume_fraction(&mut self, f: impl Into<f64>) {
        self.assumed_fraction = Some(f.into())
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
    /// Returns `None` if we cannot know, e.g. for an infinite iterator
    /// ```
    /// # use iter_progress::ProgressableIter;
    /// let mut progressor = (0..).progress().skip(120);
    /// let (state, num) = progressor.next().unwrap();
    /// assert_eq!(state.percent(), None);
    /// ```
    pub fn percent(&self) -> Option<f64> {
        self.fraction().map(|f| f * 100.)
    }

    /// Print out `msg`, but only if there has been `n` seconds since last printout. (uses
    /// `print!()`, so newline not included)
    pub fn print_every_n_sec<T: std::fmt::Display>(&self, n: f32, msg: T) {
        if self.should_do_every_n_sec(n) {
            print!("{}", msg);
        }
    }

    /// Call this function, but only every n sec (as close as possible).
    /// Could be a print statement.
    pub fn do_every_n_sec<F: Fn(&Self)>(&self, n: impl Into<f32>, f: F) {
        if self.should_do_every_n_sec(n) {
            f(self);
        }
    }

    /// If we want to do every `n` sec, should we do it now?
    pub fn should_do_every_n_sec(&self, n: impl Into<f32>) -> bool {
        let n: f32 = n.into();
        // get the secs since start as a f32
        let duration_since_start = self.duration_since_start();
        let secs_since_start: f32 = duration_since_start.as_secs() as f32
            + duration_since_start.subsec_nanos() as f32 / 1_000_000_000.0;

        match self.previous_record_tm() {
            None => {
                // This iteration is the first time, so we should print if more than `n` seconds
                // have gone past
                secs_since_start > n
            }
            Some(last_time) => {
                let last_time_offset = last_time - self.started_iterating();
                let last_time_offset: f32 = last_time_offset.as_secs() as f32
                    + last_time_offset.subsec_nanos() as f32 / 1_000_000_000.0;

                let current_step = secs_since_start / n;
                let last_step = last_time_offset / n;

                current_step.trunc() > last_step.trunc()
            }
        }
    }

    /// If we want to do every `n` items, should we do it now?
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

    /// Rolling average time to process each item this iterator is processing if it is recording
    /// that. None if it's not being recorded, or it's too soon to know (e.g. for the first item).
    pub fn rolling_average_duration(&self) -> &Option<Duration> {
        &self.rolling_average_duration
    }

    /// Rolling average number of items per second this iterator is processing if it is recording
    /// that. None if it's not being recorded, or it's too soon to know (e.g. for the first item).
    pub fn rolling_average_rate(&self) -> Option<f64> {
        self.rolling_average_duration.map(|d| 1. / d.as_secs_f64())
    }

    /// Exponential average time to process each item this iterator is processing if it is recording
    /// that. None if it's not being recorded, or it's too soon to know (e.g. for the first item).
    pub fn exp_average_duration(&self) -> &Option<Duration> {
        &self.exp_average_duration
    }

    /// Exponential average number of items per second this iterator is processing if it is recording
    /// that. None if it's not being recorded, or it's too soon to know (e.g. for the first item).
    pub fn exp_average_rate(&self) -> Option<f64> {
        self.exp_average_duration.map(|d| 1. / d.as_secs_f64())
    }

    /// If the total size is know (i.e. we know the `.fraction()`), calculate the estimated time
    /// to arrival, i.e. how long before this is finished.
    pub fn eta(&self) -> Option<Duration> {
        self.fraction()
            .map(|f| self.duration_since_start().div_f64(f) - self.duration_since_start())
    }

    /// If the total size is know (i.e. we know the `.fraction()`), calculate how long, in total,
    /// this iterator would run for. i.e. how long it's run plus how much longer it has left
    pub fn estimated_total_time(&self) -> Option<Duration> {
        self.fraction()
            .map(|f| self.duration_since_start().div_f64(f))
    }

}

pub struct OptionalProgressRecorderIter<I> {
    /// The iterator that we are iteating on
    iter: I,

    /// How many items have been seen
    count: usize,

    generate_every_count: usize,

    /// When did we start iterating
    started_iterating: Instant,

    previous_record_tm: Option<Instant>,

    rolling_average: Option<(usize, Vec<f64>)>,
    exp_average: Option<(f64, Option<Duration>)>,
    assumed_size: Option<usize>,

    _fake_now: Option<Instant>,
}

/// Wraps an iterator and keeps track of state used for `ProgressRecord`'s
pub struct ProgressRecorderIter<I>(OptionalProgressRecorderIter<I>);

impl<I> AsRef<OptionalProgressRecorderIter<I>> for ProgressRecorderIter<I> {
    fn as_ref(&self) -> &OptionalProgressRecorderIter<I> {
        &self.0
    }
}

impl<I> AsMut<OptionalProgressRecorderIter<I>> for ProgressRecorderIter<I> {
    fn as_mut(&mut self) -> &mut OptionalProgressRecorderIter<I> {
        &mut self.0
    }
}

impl<I: Iterator> ProgressRecorderIter<I> {
    /// Create a new `ProgressRecorderIter` from another iterator.
    pub fn new(iter: I) -> ProgressRecorderIter<I> {
        ProgressRecorderIter(OptionalProgressRecorderIter::new(iter, 1))
    }

    pub(crate) fn set_fake_now(&mut self, fake_now: impl Into<Option<Instant>>) {
        self.0.set_fake_now(fake_now);
    }
}

/// An iterator that records it's progress as it goes along
pub trait ProgressableIter<I> {
    fn progress(self) -> ProgressRecorderIter<I>;
}

impl<I> ProgressableIter<I> for I
where
    I: Iterator,
{
    /// Convert an iterator into a `ProgressRecorderIter`.
    fn progress(self) -> ProgressRecorderIter<I> {
        ProgressRecorderIter::new(self)
    }
}

impl<I> Iterator for ProgressRecorderIter<I>
where
    I: Iterator,
{
    type Item = (ProgressRecord, <I as Iterator>::Item);

    #[inline]
    fn next(&mut self) -> Option<(ProgressRecord, <I as Iterator>::Item)> {
        self.0.iter.next().map(|a| {
            let fake_now = std::mem::take(&mut self.0._fake_now);
            // we know there is always a record generated
            (self.0.generate_record(fake_now).unwrap(), a)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.iter.count()
    }
}

impl<I: Iterator> OptionalProgressRecorderIter<I> {
    pub fn new(iter: I, generate_every_count: usize) -> OptionalProgressRecorderIter<I> {
        OptionalProgressRecorderIter {
            iter,
            count: 0,
            generate_every_count,
            started_iterating: Instant::now(),
            previous_record_tm: None,
            rolling_average: None,
            exp_average: None,
            assumed_size: None,
            _fake_now: None,
        }
    }

    /// Set the desired size of the rolling average window calculation (if any). `None` to
    /// disable.
    /// Larger values slow down each iteration (since the rolling average is calculated each
    /// iteration).
    pub fn with_rolling_average(self, size: impl Into<Option<usize>>) -> Self {
        let mut res = self;
        res.rolling_average = size.into().map(|size| (size, vec![0.; size]));
        res
    }

    /// Set the desired exponential rate
    /// 0.001 is a good value.
    pub fn with_exp_average(self, rate: impl Into<Option<f64>>) -> Self {
        let mut res = self;
        res.exp_average = rate.into().map(|rate| (rate, None));
        res
    }

    /// Add an 'assumed size' to this iterator. If the iterator doesn't return an exact value for
    /// `.size_hint()`, you can use this to override
    /// the `.size_hint()` from the iterator will override this if it returns an exact size (i.e.
    /// `.size_hint().1 == Some(...size_hint().0).
    /// Set to `None` to undo this.
    pub fn assume_size(self, size: impl Into<Option<usize>>) -> Self {
        let mut new = self;
        new.assumed_size = size.into();
        new
    }

    /// Calculate the current `ProgressRecord` for where we are now.
    fn generate_record(&mut self, fake_now: Option<Instant>) -> Option<ProgressRecord> {
        self.count += 1;
        if self.count % self.generate_every_count != 0 {
            return None;
        }

        let now = fake_now.unwrap_or_else(|| Instant::now());

        let exp_average_rate = if let Some((rate, last)) = self.exp_average {
            if let Some(previous_tm) = self.previous_record_tm {
                let this_duration = now - previous_tm;
                let current_ema = match last {
                    None => this_duration,
                    Some(last) => this_duration.mul_f64(rate) + last.mul_f64(1. - rate),
                };
                self.exp_average = Some((rate, Some(current_ema)));
                Some(current_ema)
            } else {
                None
            }
        } else {
            None
        };

        let rolling_average_duration = match &mut self.rolling_average {
            None => None,
            Some((size, values)) => {
                if let Some(previous_tm) = self.previous_record_tm {
                    let this_duration = (now - previous_tm).as_secs_f64();
                    values[self.count % *size] = this_duration;
                    if self.count < *size {
                        // We haven't filled up the buffer yet
                        Some(Duration::from_secs_f64(
                            values[0..=self.count].iter().sum::<f64>() / (self.count as f64),
                        ))
                    } else {
                        Some(Duration::from_secs_f64(
                            values.iter().sum::<f64>() / (*size as f64),
                        ))
                    }
                } else {
                    None
                }
            }
        };

        let res = ProgressRecord {
            num: self.count,
            iterating_for: now - self.started_iterating,
            size_hint: self.iter.size_hint(),
            assumed_size: self.assumed_size,
            assumed_fraction: None,
            started_iterating: self.started_iterating,
            previous_record_tm: self.previous_record_tm,
            rolling_average_duration,
            exp_average_duration: exp_average_rate,
        };

        self.previous_record_tm = Some(now);

        Some(res)
    }

    /// Returns referend to the inner iterator
    pub fn inner(&self) -> &I {
        &self.iter
    }

    /// Gets the original iterator back, consuming this.
    pub fn into_inner(self) -> I {
        self.iter
    }

    pub(crate) fn set_fake_now(&mut self, fake_now: impl Into<Option<Instant>>) {
        self._fake_now = fake_now.into();
    }
}

pub trait OptionalProgressableIter<I: Iterator> {
    fn optional_progress(self, generate_every_count: usize) -> OptionalProgressRecorderIter<I>;
}

impl<I> OptionalProgressableIter<I> for I
where
    I: Iterator,
{
    /// Convert an iterator into an `OptionalProgressRecorderIter`.
    fn optional_progress(self, generate_every_count: usize) -> OptionalProgressRecorderIter<I> {
        OptionalProgressRecorderIter::new(self, generate_every_count)
    }
}

impl<I: Iterator> Iterator for OptionalProgressRecorderIter<I> {
    type Item = (Option<ProgressRecord>, <I as Iterator>::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let fake_now = std::mem::take(&mut self._fake_now);
        self.iter.next().map(|a| (self.generate_record(fake_now), a))
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
