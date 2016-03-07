use std::iter::Iterator;

pub struct ProgressRecord {
    num: usize,
}

pub struct ProgressRecorderIter<I> {
    iter: I,
    count: usize,
}

impl<I> ProgressRecorderIter<I> {
    pub fn new(iter: I) -> ProgressRecorderIter<I> {
        ProgressRecorderIter{ iter: iter, count: 0 }
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
        let rec = ProgressRecord{ num: 0 };
        self.iter.next().map(|a| {
            let ret = (rec, a);
            ret
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

        let vec: Vec<u8> = vec![0, 1, 2, 3, 4];
        let mut it = vec.iter().progress();
        let next = it.next();
        assert!(next.is_some());
        let (state, &val) = next.unwrap();
        assert_eq!(val, 0);
    }

}

