#![deny(missing_docs, rust_2018_idioms)]

//! An extension trait for date-like types to allow computing "calendar durations" between dates.
//!
//! A calendar duration is a duration which takes into account the specific calendar dates
//! involved. This recognizes that not all months have the same number of days, so saying "one
//! month ago" means a different amount of absolute time depending of which date it's in reference
//! to, and may not even be a well-defined date. The same goes for years: some years have 365 days,
//! but some have 366, and so "4 years ago" may not have a well-defined meaning, depending on which
//! years those are.
//!
//! Note that this crate is only concerned with whole days. It does not account for leap-seconds or
//! timezone changes, and so the trait is only implemented for date-like types, not date-time ones.
//!
//! This crate comes with implementations for two types:
//!   - [`chrono::NaiveDate`] which can be enabled by compiling with the `chrono` feature.
//!   - [`time::Date`] which can be enabled by compiling with the `time` feature.
//!
//! By default, neither is enabled, and because of Rust's prohibition on implementations of foreign
//! traits for foreign types, the trait by itself is not useful without one of these
//! implementations. So you'll need to use it in your Crates.toml as
//! ```cargo
//! calendar_duration = { version = "$current_version_here", features = ["chrono"] }
//! ```
//! (or `features = ["time"]` if you're using that crate.)

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
                    (2, 29) | (2, 30) | (2, 31) => Self::from_ymd(y, 3, 1).unwrap(),
                    (_, 31) => {
                        if m == 12 {
                            m = 1;
                            y += 1;
                        } else {
                            m += 1;
                        }
                        Self::from_ymd(y, m, 30).unwrap()
                    }
                    _ => panic!("constructing a date for ({y},{m},{d}) failed for unknown reason"),
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

        CalendarDuration { years, months, days }
    }
}

/// A calendar duration is a duration which takes into account the calendar dates involved. See the
/// [module level documentation](crate) for more info.
///
/// Calendar duration includes the number of years, months, and days.
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

        if !any {
            f.write_str("same day")?
        }

        Ok(())
    }
}

#[cfg(test)]
macro_rules! tests {
    ($ctor:expr) => {
        #[test]
        fn it_works() {
            let a = $ctor(2020, 4, 8);
            let b = $ctor(1988, 6, 16);
            let c = a.calendar_duration_from(b);
            assert_eq!(c.to_string(), "31 years, 9 months, 23 days");
        }

        #[test]
        fn same() {
            assert_eq!("same day",
                $ctor(1999, 12, 31)
                    .calendar_duration_from(
                        $ctor(1999, 12, 31))
                    .to_string());
        }

        #[test]
        fn leapyear1() {
            assert_eq!("1 year",
                $ctor(2005, 3, 1)
                    .calendar_duration_from(
                        $ctor(2004, 2, 29))
                    .to_string());
        }

        #[test]
        fn leapyear2() {
            assert_eq!("1 year",
                $ctor(2005, 3, 1)
                    .calendar_duration_from(
                        $ctor(2004, 3, 1))
                    .to_string());
        }

        #[test]
        fn straddle_30_days_month() {
            assert_eq!("2 months",
                $ctor(2000, 7, 31)
                    .calendar_duration_from(
                        $ctor(2000, 5, 31))
                    .to_string());
        }

        #[test]
        fn not_all_months_are_31_days() {
            let start = $ctor(2000, 8, 31);
            let mut earlier = $ctor(2000, 6, 30);

            assert_eq!("2 months, 1 day", start.calendar_duration_from(earlier).to_string());

            // Next day goes to 2000-07-01 because June has 30 days.
            earlier = earlier.succ();

            // So we never get exactly "2 months".
            assert_eq!("1 month, 30 days", start.calendar_duration_from(earlier).to_string());
        }

        #[test]
        fn test_feb30() {
            let mut start = $ctor(2024, 12, 29);
            let later = $ctor(2025, 3, 15);

            assert_eq!("2 months, 14 days", start.calendar_duration_from(later).to_string());

            start = start.succ(); // 2024-12-30
            assert_eq!("2 months, 14 days", start.calendar_duration_from(later).to_string());

            start = start.succ(); // 2024-12-31
            assert_eq!("2 months, 14 days", start.calendar_duration_from(later).to_string());

            start = start.succ(); // 2025-01-01
            assert_eq!("2 months, 14 days", start.calendar_duration_from(later).to_string());

            start = start.succ(); // 2025-01-02
            assert_eq!("2 months, 13 days", start.calendar_duration_from(later).to_string());
        }
    }
}

#[cfg(feature = "chrono")]
mod chrono_impl {
    use super::*;
    use chrono::{Datelike, NaiveDate};

    impl CalendarDurationExt for chrono::NaiveDate {
        fn ymd(self) -> (i32, u8, u8) {
            (self.year(),
                u8::try_from(self.month()).expect("month out of bounds"),
                u8::try_from(self.day()).expect("day out of bounds"))
        }

        fn from_ymd(y: i32, m: u8, d: u8) -> Option<Self> {
            NaiveDate::from_ymd_opt(y, u32::from(m), u32::from(d))
        }

        fn succ(self) -> Self {
            NaiveDate::succ_opt(&self).expect("date out of range")
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        tests!(NaiveDate::from_ymd);
    }
}


#[cfg(feature = "time")]
mod time_impl {
    use super::*;
    use time::Date;

    impl CalendarDurationExt for time::Date {
        fn ymd(self) -> (i32, u8, u8) {
            let (y, m, d) = self.to_calendar_date();
            (y, m as u8, d)
        }

        fn from_ymd(y: i32, m: u8, d: u8) -> Option<Self> {
            Date::from_calendar_date(y, time::Month::try_from(m).ok()?, d).ok()
        }

        fn succ(self) -> Self {
            self.next_day().expect("cannot increment max date")
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        tests!(|y, m, d| {
            let month = time::Month::try_from(m).expect("invalid month");
            Date::from_calendar_date(y, month, d).expect("failed to construct Date")
        });
    }
}
