use super::*;

#[test]
fn test_simple() {
    use super::ProgressableIter;
    use std::thread::sleep;
    use std::time::Duration;

    let vec: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut progressor = vec.iter().progress();

    sleep(Duration::from_millis(500));
    let (state, _) = progressor.next().unwrap();
    // It'll always print on the first one
    assert_eq!(state.should_do_every_n_items(2), true);
    assert_eq!(state.should_do_every_n_items(3), true);
    assert_eq!(state.should_do_every_n_items(5), true);
    assert_eq!((state.rate() * 100.).round(), 200.);
    // First run, so there should be nothing here
    assert!(state.previous_record_tm().is_none());

    assert_eq!(state.should_do_every_n_sec(1.), false);
    assert_eq!(state.should_do_every_n_sec(2.), false);
    assert_eq!(state.should_do_every_n_sec(0.3), true);

    sleep(Duration::from_millis(500));

    let (state, _) = progressor.next().unwrap();
    assert_eq!(state.should_do_every_n_items(2), false);
    assert_eq!(state.should_do_every_n_items(3), false);
    assert_eq!(state.should_do_every_n_items(5), false);
    assert_eq!(state.rate().round(), 2.);
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
    assert_eq!(state.should_do_every_n_items(2), true);
    assert_eq!(state.should_do_every_n_items(3), false);
    assert_eq!(state.should_do_every_n_items(5), false);
    assert_eq!(state.rate().round(), 2.);
    assert_eq!(state.should_do_every_n_sec(1.), false);
    assert_eq!(state.should_do_every_n_sec(2.), false);
    assert_eq!(state.should_do_every_n_sec(0.8), false);
    assert_eq!(state.should_do_every_n_sec(1.5), true);

    sleep(Duration::from_millis(500));
    let (state, _) = progressor.next().unwrap();
    assert_eq!(state.should_do_every_n_items(2), false);
    assert_eq!(state.should_do_every_n_items(3), true);
    assert_eq!(state.should_do_every_n_items(5), false);
    assert_eq!(state.rate().round(), 2.);
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

#[test]
fn assume_fraction1() {
    use super::ProgressableIter;

    let vec: Vec<u8> = vec![0, 1, 2, 3, 4];
    let mut progressor = vec.iter().progress();

    let (mut state, _) = progressor.next().unwrap();
    assert_eq!(state.fraction(), Some(0.2));
    assert_eq!(state.percent(), Some(20.));
    state.assume_fraction(0.5);
    assert_eq!(state.fraction(), Some(0.5));
    assert_eq!(state.percent(), Some(50.));

    let (state, _) = progressor.next().unwrap();
    assert_eq!(state.fraction(), Some(0.4));
    assert_eq!(state.percent(), Some(40.));

    let mut progressor = (0..).progress();

    let (mut state, val) = progressor.next().unwrap();
    assert_eq!(val, 0);
    assert_eq!(state.fraction(), None);
    state.assume_fraction(0.2);
    assert_eq!(state.fraction(), Some(0.2));
    let (state, val) = progressor.next().unwrap();
    assert_eq!(val, 1);
    assert_eq!(state.fraction(), None);
}

#[test]
fn optional() {
    let vec: Vec<u8> = vec![0, 1, 2, 3, 4];
    let progressed_iterator = vec.iter().optional_progress(3).collect::<Vec<_>>();
    dbg!(&progressed_iterator);
    assert!(progressed_iterator[0].0.is_none());
    assert!(progressed_iterator[1].0.is_none());
    assert!(progressed_iterator[2].0.is_some());
    assert!(progressed_iterator[3].0.is_none());
    assert!(progressed_iterator[4].0.is_none());
}
