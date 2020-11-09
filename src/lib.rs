#![deny(missing_docs, rust_2018_idioms)]

//! An extension trait for date-like types to allow computing "calendar durations" between dates.
//!
//! A calendar duration is a duration which takes into account the specific calendar dates
//! involved. This recognizes that not all months have the same number of days, so saying "one
//! month ago" can mean a different date depending of which date it's in reference to. The same
//! goes for years: some years have 365 days, but some have 366, and so "4 years ago" can mean a
//! different date depending on which years those are.
//!
//! This crate comes with implementations for two types:
//!   - [`chrono::NaiveDate`] which can be enabled by compiling with the `chrono` feature.
//!   - [`time::Date`] which can be enabled by compiling with the `time` feature.

use std::cmp::Ordering;

/// Extension trait to allow computing a "calendar duration" from two dates.
/// 
/// See [`CalendarDuration`] for more info.
pub trait CalendarDurationExt: Sized + Ord + Copy {
    /// Return a 3-tuple of the year, month, and day (one-based) for the date.
    fn ymd(self) -> (i32, u8, u8);

    /// Construct a date from the given year, month, and date, if such a date is valid.
    fn from_ymd(y: i32, m: u8, d: u8) -> Option<Self>;

    /// Construct a date from the given year, month, and date; or the next day if such date is not
    /// valid (either leap year or 30/31 day month difference).
    fn from_ymd_or_next(mut y: i32, mut m: u8, d: u8) -> Self {
        Self::from_ymd(y, m, d)
            .unwrap_or_else(|| {
                match (m, d) {
                    (2, 29) => Self::from_ymd(y, 3, 1).unwrap(),
                    (_, 31) => {
                        if m == 12 {
                            m = 1;
                            y += 1;
                        } else {
                            m += 1;
                        }
                        Self::from_ymd(y, m, 30).unwrap()
                    }
                    _ => panic!("constructing a date for ({},{},{}) failed for unknown reason",
                            y, m, d),
                }
            })
    }

    /// Return the date for the next day from the given one.
    fn succ(self) -> Self;

    /// Compute the calendar duration difference from the other date.
    fn calendar_duration_from(self, other: Self) -> CalendarDuration {
        let (later, mut earlier) = if self > other {
            (self, other)
        } else {
            (other, self)
        };

        let order = self.cmp(&other);

        let (mut y, mut m, d) = earlier.ymd();
        let mut years = 0u32;
        loop {
            let next = Self::from_ymd_or_next(y + 1, m, d);
            if later < next {
                break;
            }
            years += 1;
            y += 1;
            earlier = next;
        }

        let mut months = 0;
        loop {
            let mut next_m = m + 1;
            let mut next_y = y;
            if next_m == 13 {
                next_m = 1;
                next_y += 1;
            }

            let next = Self::from_ymd_or_next(next_y, next_m, d);
            if later < next {
                break;
            }

            months += 1;
            y = next_y;
            m = next_m;
            earlier = next;
        }

        let mut days = 0;
        while later > earlier {
            days += 1;
            earlier = earlier.succ();
        }

        CalendarDuration { order, years, months, days }
    }
}

/// A calendar duration is a duration which takes into account the calendar dates involved. See the
/// [module level documentation](crate) for more info.
///
/// Calendar duration includes the number of years, months, and days, and an Ordering indicating
/// whether the difference is in the future or the past.
///
/// It includes a [`Display`](std::fmt::Display) implementation which formats the duration nicely
/// in English.
#[derive(Debug, Clone)]
pub struct CalendarDuration {
    /// Number of whole years of duration.
    pub years: u32,

    /// Number of whole months in addition to the [`years`](Self::years).
    pub months: u8,

    /// Number of whole days in addition to the [`months`](Self::months) and
    /// [`years`](Self::years).
    pub days: u8,

    /// Indicates whether the duration is forwards or backwards in time.
    ///
    /// [`Ordering::Greater`] means the duration is into the past, and [`Ordering::Less`] means the
    /// duration is into the future. [`Ordering::Equal`] means the duration is less than one full
    /// day, and all other fields must be zero.
    pub order: Ordering,
}

impl std::fmt::Display for CalendarDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut any = false;
        if self.years > 0 {
            if self.years > 1 {
                write!(f, "{} years", self.years)?;
            } else {
                f.write_str("1 year")?;
            }
            any = true;
        }

        if self.months > 0 {
            if any {
                f.write_str(", ")?;
            }
            if self.months > 1 {
                write!(f, "{} months", self.months)?;
            } else {
                f.write_str("1 month")?;
            }
            any = true;
        }

        if self.days > 0 {
            if any {
                f.write_str(", ")?;
            }
            if self.days > 1 {
                write!(f, "{} days", self.days)?;
            } else {
                f.write_str("1 day")?;
            }
            any = true;
        }

        if any {
            match self.order {
                Ordering::Greater => {
                    f.write_str(" ago")?;
                }
                Ordering::Less => {
                    f.write_str(" to go")?;
                }
                Ordering::Equal => panic!("unexpected equal ordering with nonzero duration"),
            }
        } else {
            f.write_str("same day")?
        }

        Ok(())
    }
}

#[cfg(feature = "chrono")]
mod chrono_impl {
    use super::*;
    use chrono::{Datelike, NaiveDate};

    impl CalendarDurationExt for chrono::NaiveDate {
        fn ymd(self) -> (i32, u8, u8) {
            use std::convert::TryFrom;
            (self.year(),
                u8::try_from(self.month()).expect("month out of bounds"),
                u8::try_from(self.day()).expect("day out of bounds"))
        }

        fn from_ymd(y: i32, m: u8, d: u8) -> Option<Self> {
            NaiveDate::from_ymd_opt(y, u32::from(m), u32::from(d))
        }

        fn succ(self) -> Self {
            NaiveDate::succ(&self)
        }
    }

}

#[cfg(feature = "time")]
mod time_impl {
    use super::*;
    use time::Date;

    impl CalendarDurationExt for time::Date {
        fn ymd(self) -> (i32, u8, u8) {
            self.as_ymd()
        }

        fn from_ymd(y: i32, m: u8, d: u8) -> Option<Self> {
            Date::try_from_ymd(y, m, d).ok()
        }

        fn succ(self) -> Self {
            self.next_day()
        }
    }
}

#[cfg(all(feature = "chrono", test))]
mod chrono_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn it_works() {
        let a = NaiveDate::from_ymd(2020, 4, 8);
        let b = NaiveDate::from_ymd(1988, 6, 16);
        let c = a.calendar_duration_from(b);
        assert_eq!(c.to_string(), "31 years, 9 months, 23 days ago");
    }

    #[test]
    fn same() {
        assert_eq!("same day",
            NaiveDate::from_ymd(1999, 12, 31)
                .calendar_duration_from(
                    NaiveDate::from_ymd(1999, 12, 31))
                .to_string());
    }

    #[test]
    fn leapyear1() {
        assert_eq!("1 year ago",
            NaiveDate::from_ymd(2005, 3, 1)
                .calendar_duration_from(
                    NaiveDate::from_ymd(2004, 2, 29))
                .to_string());
    }
    
    #[test]
    fn leapyear2() {
        assert_eq!("1 year ago",
            NaiveDate::from_ymd(2005, 3, 1)
                .calendar_duration_from(
                    NaiveDate::from_ymd(2004, 3, 1))
                .to_string());
    }

    #[test]
    fn straddle_30_days_month() {
        assert_eq!("2 months",
            NaiveDate::from_ymd(2000, 7, 31)
                .calendar_duration_from(
                    NaiveDate::from_ymd(2000, 5, 31))
                .to_string());
    }

    #[test]
    fn not_all_months_are_31_days() {
        let start = NaiveDate::from_ymd(2000, 8, 31);
        let mut earlier = NaiveDate::from_ymd(2000, 6, 30);

        assert_eq!("2 months, 1 day", start.calendar_duration_from(earlier).to_string());

        // Next day goes to 2000-07-01 because June has 30 days.
        earlier = earlier.succ();

        // So we never get exactly "2 months".
        assert_eq!("1 month, 30 days", start.calendar_duration_from(earlier).to_string());
    }
}

#[cfg(all(feature = "time", test))]
mod time_tests {
    use super::*;

    #[test]
    fn it_works() {
        let a = time::date!(2020-04-08);
        let b = time::date!(1988-06-16);
        let c = a.calendar_duration_from(b);
        assert_eq!(c.to_string(), "31 years, 9 months, 23 days ago");
    }

    #[test]
    fn same() {
        assert_eq!("same day",
            time::date!(1999-12-31)
                .calendar_duration_from(
                    time::date!(1999-12-31))
                .to_string());
    }

    #[test]
    fn leapyear1() {
        assert_eq!("1 year ago",
            time::date!(2005-03-01)
                .calendar_duration_from(
                    time::date!(2004-02-29))
                .to_string());
    }
    
    #[test]
    fn leapyear2() {
        assert_eq!("1 year ago",
            time::date!(2005-03-01)
                .calendar_duration_from(
                    time::date!(2004-03-01))
                .to_string());
    }

    #[test]
    fn straddle_30_days_month() {
        assert_eq!("2 months",
            time::date!(2000-07-31)
                .calendar_duration_from(
                    time::date!(2000-05-31))
                .to_string());
    }

    #[test]
    fn not_all_months_are_31_days() {
        let start = time::date!(2000-08-31);
        let mut earlier = time::date!(2000-06-30);

        assert_eq!("2 months, 1 day", start.calendar_duration_from(earlier).to_string());

        // Next day goes to 2000-07-01 because June has 30 days.
        earlier = earlier.succ();

        // So we never get exactly "2 months".
        assert_eq!("1 month, 30 days", start.calendar_duration_from(earlier).to_string());
    }
}
