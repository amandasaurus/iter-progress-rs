extern crate time;

use std::iter::Iterator;

use time::{now_utc, Tm, Duration};

pub struct ProgressRecord {
    num: usize,
    iterating_for: Duration,

}

impl ProgressRecord {
    pub fn message(&self) -> String {
        format!("Have seen {} items and been iterating for {}", self.num, self.iterating_for.num_seconds())
    }

    pub fn rate(&self) -> f32 {
        // number of items per second
        0.
    }

    pub fn fraction(&self) -> Option<f32> {
        None
    }

    pub fn percent(&self) -> Option<f32> {
        match self.fraction() {
            None => { None }
            Some(f) => { Some(f * 100.0) }
        }
    }

}

pub struct ProgressRecorderIter<I> {
    iter: I,
    count: usize,
    started_iterating: Tm,
}

impl<I> ProgressRecorderIter<I> {
    pub fn new(iter: I) -> ProgressRecorderIter<I> {
        ProgressRecorderIter{ iter: iter, count: 0, started_iterating: now_utc() }
    }

    fn generate_record(&mut self) -> ProgressRecord {
        self.count += 1;
        ProgressRecord{ num: self.count, iterating_for: now_utc() - self.started_iterating }
    }

}

pub trait ProgressableIter<I> {
    fn progress(self) -> ProgressRecorderIter<I>;
}

impl<I> ProgressableIter<I> for I where I: Iterator {
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
        sleep_ms(1500);
        let (state, val) = progressor.next().unwrap();
        assert_eq!(state.message(), "Have seen 1 items and been iterating for 1")
    }

}

