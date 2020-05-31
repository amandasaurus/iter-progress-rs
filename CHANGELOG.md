## Unreleased

* State gets `.assume_fraction(…)` for when you want to force a specific fraction value

## v0.6.0 (2020-01-02)

* Replace `.recent_rate()` with exponential & rolling average functionality
* Remove `.message()`, please write your own message
* Added `.assume_total(usize)` functionality, allowing you to use this total as final fall back
* Use `f64` instead of `f32` in some place (e.g. `.fraction()`)
* Added `.eta()` & `.estimated_total_time()` methods to retrieve that when the
  total is estimatable.
* Minor internal code clean ups

## v0.5.0 (2019-09-28)

* Change licence to [Affero GPL licence](LICENCE)
* Minor documentation improvements

## v0.4.0 (2017-02-10)

### Features

* Add `.into_inner()` to get the inner iter back out ([940a4626](940a4626))


## v0.2.0 (2016-04-11)

### Features

* Add `should_print_every_sec` method ([fd554c55](fd554c55))
* ProgressRecords now keep track of when iteration started ([9e7fb771](9e7fb771))
* Keep track of previous timestamp of records ([bb2208f6](bb2208f6))
* Keep track of recent rate, rather than global rate ([46e43adb](46e43adb))


##  (2016-03-08)

### Features

* Accept any Display-able thing for a message ([68104ea2](68104ea2))
* Improve default message ([f08198d4](f08198d4))
* Add `.fraction()` method which tells you how far you are along ([fa799e50](fa799e50))
* Add print_every method ([7741193d](7741193d))
* Add simple time tracking ([8e8d5ad4](8e8d5ad4))
