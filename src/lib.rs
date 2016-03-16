extern crate time;

use std::iter::Iterator;

use time::{now_utc, Tm, Duration};

pub struct ProgressRecord {

    /// How many elements before this
    num: usize,

    /// How long since we started iterating.
    iterating_for: Duration,
    size_hint: (usize, Option<usize>),

    recent_rate: f32,

}

impl ProgressRecord {
    /// Returns a basic log message of where we are now. You can construct this yourself, but this
    /// is a helpful convience method.
    pub fn message(&self) -> String {
        format!("{} - Seen {} Rate {}/sec", self.duration_since_start().num_seconds(), self.num_done(), self.recent_rate())
    }

    /// Duration since iteration started
    pub fn duration_since_start(&self) -> Duration {
        self.iterating_for
    }

    /// Number of items we've generated so far
    pub fn num_done(&self) -> usize {
        self.num
    }

    /// Prints a basic message
    pub fn print_message(&self) {
        println!("{}", self.message());
    }

    /// Number of items per second
    pub fn rate(&self) -> f32 {
        // number of items per second
        (self.num_done() as f32) / (self.duration_since_start().num_seconds() as f32)
    }

    /// None if we don't know how much we've done (as a fraction), otherwise a value form 0 to 1
    /// for what fraction along we are.
    pub fn fraction(&self) -> Option<f32> {
        if self.is_size_known() {
            let remaining = self.size_hint.0;
            let done = self.num_done();
            Some(( done as f32 ) / ( (remaining+done) as f32 ))
        } else {
            None
        }
    }

    /// None if we don't know how much we've done, otherwise value for 0 to 100 representing how
    /// far along as a percentage we are.
    pub fn percent(&self) -> Option<f32> {
        match self.fraction() {
            None => { None }
            Some(f) => { Some(f * 100.0) }
        }
    }

    /// If we want to print every `n` items, should we print now?
    pub fn should_print_every_items(&self, n: usize) -> bool {
        (self.num_done() - 1) % n == 0
    }

    /// Print out `msg`, but only if there has been `n` items.
    /// Often you want to print out a debug message every 1,000 items or so. This function does
    /// that.
    pub fn print_every_sec<T: std::fmt::Display>(&self, n: usize, msg: T) {
        if self.should_print_every_items(n) {
            print!("{}", msg);
        }
    }

    /// If we want to print every `n` sec, should we print now?
    pub fn should_print_every_sec(&self, n: f32) -> bool {
        //(self.num_done() - 1) % n == 0
        false
    }

    /// Print out `msg`, but only if there has been `n` items.
    /// Often you want to print out a debug message every 1,000 items or so. This function does
    /// that.
    pub fn print_every<T: std::fmt::Display>(&self, n: usize, msg: T) {
        if self.should_print_every_items(n) {
            print!("{}", msg);
        }
    }

    /// Does the size_hint tell us exactly how many items are left? False iff there is some
    /// ambiguity/unknown
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
    started_iterating: Tm,

    /// Keeps track of recent times
    recent_times: Vec<Tm>
}

impl<I: Iterator> ProgressRecorderIter<I> {
    /// Create a new `ProgressRecorderIter` from another iterator.
    pub fn new(iter: I) -> ProgressRecorderIter<I> {
        ProgressRecorderIter{ iter: iter, count: 0, started_iterating: now_utc(), recent_times: Vec::with_capacity(5) }
    }

    /// Calculate the current `ProgressRecord` for where we are now.
    fn generate_record(&mut self) -> ProgressRecord {
        let now = now_utc();
        self.recent_times.push(now);
        while self.recent_times.len() > 100 {
            self.recent_times.remove(0);
        }

        //println!("\n");
        //println!("Recents {:?} now {:?}", self.recent_times.iter().map(|&d| { (now - d).num_seconds() }).collect::<Vec<_>>(), now);
        let recent_rate = match self.recent_times.get(0) { None => { ::std::f32::INFINITY }, Some(&first) => {
            //println!("{} {}", self.recent_times.len() as f32, (now - first).num_seconds() as f32);
            (self.recent_times.len() as f32 ) / ((now - first).num_seconds() as f32)
        }, };

        self.count += 1;
        ProgressRecord{ num: self.count, iterating_for: now - self.started_iterating, size_hint: self.iter.size_hint(), recent_rate: recent_rate }
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
        use std::thread::sleep_ms;
        use time::Duration;

        let vec: Vec<u8> = vec![0, 1, 2, 3, 4];
        let mut progressor = vec.iter().progress();

        sleep_ms(500);
        let (state, _) = progressor.next().unwrap();
        assert_eq!(state.message(), "0 - Seen 1 Rate inf/sec");
        // It'll always print on the first one
        assert_eq!(state.should_print_every_items(2), true);
        assert_eq!(state.should_print_every_items(3), true);
        assert_eq!(state.should_print_every_items(5), true);
        assert_eq!(state.rate(), ::std::f32::INFINITY);
        assert_eq!(state.recent_rate(), ::std::f32::INFINITY);

        sleep_ms(500);
        let (state, _) = progressor.next().unwrap();
        assert_eq!(state.message(), "1 - Seen 2 Rate inf/sec");
        assert_eq!(state.should_print_every_items(2), false);
        assert_eq!(state.should_print_every_items(3), false);
        assert_eq!(state.should_print_every_items(5), false);
        assert_eq!(state.rate(), 2.);
        //assert_eq!(state.recent_rate(), 2.);

        sleep_ms(500);
        let (state, _) = progressor.next().unwrap();
        assert_eq!(state.message(), "1 - Seen 3 Rate 3/sec");
        assert_eq!(state.should_print_every_items(2), true);
        assert_eq!(state.should_print_every_items(3), false);
        assert_eq!(state.should_print_every_items(5), false);
        assert_eq!(state.rate(), 3.);
        assert_eq!(state.recent_rate(), 3.);

        sleep_ms(500);
        let (state, _) = progressor.next().unwrap();
        assert_eq!(state.message(), "2 - Seen 4 Rate 4/sec");
        assert_eq!(state.should_print_every_items(2), false);
        assert_eq!(state.should_print_every_items(3), true);
        assert_eq!(state.should_print_every_items(5), false);
        assert_eq!(state.rate(), 2.);
        assert_eq!(state.recent_rate(), 4.);
    }

    #[test]
    fn test_size_hint() {
        use super::ProgressableIter;
        use time::Duration;

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

