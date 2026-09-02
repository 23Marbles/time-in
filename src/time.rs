use std::fmt::Display;

use chrono::{DateTime, Days, Months, NaiveDateTime, TimeDelta, TimeZone};
use num_integer::Integer;

pub enum TimeValue {
    Duration(NaiveDuration),
    DateTime(NaiveDateTime),
}

/// Includes measurement for changeable lengths of time months
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct NaiveDuration {
    is_neg: bool,
    months: u32,
    days: u64,
    standard: TimeDelta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeUnitDur {
    pub duration: i64,
    pub time_kind: TimeKind,
}

impl NaiveDuration {
    #[must_use]
    pub fn is_neg(&self) -> bool {
        self.is_neg
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.months == 0 && self.days == 0 && self.standard.is_zero()
    }

    pub fn datetime_checked_add<Tz: TimeZone>(
        self,
        mut datetime: DateTime<Tz>,
    ) -> Option<DateTime<Tz>> {
        let Self {
            months,
            days,
            standard,
            is_neg,
        } = self;

        datetime = datetime.checked_add_signed(standard)?;

        let months = Months::new(months);
        let days = Days::new(days);

        if is_neg {
            datetime = datetime.checked_sub_months(months)?;
            datetime = datetime.checked_sub_days(days)?;
        } else {
            datetime = datetime.checked_add_months(months)?;
            datetime = datetime.checked_add_days(days)?;
        }

        Some(datetime)
    }

    pub fn from_time_unit_durs<'a>(durs: impl Iterator<Item = &'a TimeUnitDur>) -> Option<Self> {
        let mut this = Self::default();

        for d in durs {
            match d.time_kind {
                TimeKind::Secs => this = this.add_seconds(d.duration)?,
                TimeKind::Mins => this = this.add_minutes(d.duration)?,
                TimeKind::Hours => this = this.add_hours(d.duration)?,
                TimeKind::Days => this = this.add_days(d.duration.try_into().ok()?)?,
                TimeKind::Weeks => this = this.add_weeks(d.duration.try_into().ok()?)?,
                TimeKind::Months => this = this.add_months(d.duration.try_into().ok()?)?,
                TimeKind::Years => this = this.add_years(d.duration.try_into().ok()?)?,
            }
        }

        Some(this)
    }

    #[must_use]
    pub fn flip_sign(self) -> Self {
        Self {
            is_neg: !self.is_neg,
            standard: -self.standard,
            ..self
        }
    }

    #[must_use]
    pub fn add_years(mut self, years: u32) -> Option<Self> {
        self.months = self.months.checked_add(years.checked_mul(12)?)?;

        Some(self)
    }

    #[must_use]
    pub fn add_months(mut self, months: u32) -> Option<Self> {
        self.months = self.months.checked_add(months)?;

        Some(self)
    }

    #[must_use]
    pub fn add_weeks(mut self, weeks: u64) -> Option<Self> {
        self.days = self.days.checked_add(weeks.checked_mul(7)?)?;

        Some(self)
    }

    #[must_use]
    pub fn add_days(mut self, days: u64) -> Option<Self> {
        self.days = self.days.checked_add(days)?;

        Some(self)
    }

    #[must_use]
    pub fn add_hours(mut self, hours: i64) -> Option<Self> {
        self.standard = self.standard.checked_add(&TimeDelta::try_hours(hours)?)?;

        Some(self)
    }

    #[must_use]
    pub fn add_minutes(mut self, minutes: i64) -> Option<Self> {
        self.standard = self
            .standard
            .checked_add(&TimeDelta::try_minutes(minutes)?)?;

        Some(self)
    }

    #[must_use]
    pub fn add_seconds(mut self, seconds: i64) -> Option<Self> {
        self.standard = self
            .standard
            .checked_add(&TimeDelta::try_seconds(seconds)?)?;

        Some(self)
    }
}

pub const WEEK_IN_SECS: i64 = DAY_IN_SECS * 7;
pub const DAY_IN_SECS: i64 = HOUR_IN_SECS * 24;
pub const HOUR_IN_SECS: i64 = MIN_IN_SECS * 60;
pub const MIN_IN_SECS: i64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimeKind {
    Secs,
    Mins,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

#[deprecated]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeDeltaFormatter {
    secs: i64,
}

#[expect(deprecated)]
impl Display for TimeDeltaFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn with_suffix_if_not_zero(num: i64, suffix: &str) -> Option<String> {
            if num == 0 {
                return None;
            }

            Some(format!("{num} {suffix}{}", if num == 1 { "" } else { "s" }))
        }

        let rem = self.secs;

        #[cfg(feature = "show_weeks")]
        let (weeks, rem) = rem.div_rem(&WEEK_IN_SECS);
        let (days, rem) = rem.div_rem(&DAY_IN_SECS);
        let (hours, rem) = rem.div_rem(&HOUR_IN_SECS);
        let (mins, secs) = rem.div_rem(&MIN_IN_SECS);

        let iter = [
            #[cfg(feature = "show_weeks")]
            with_suffix_if_not_zero(weeks, "week"),
            with_suffix_if_not_zero(days, "day"),
            with_suffix_if_not_zero(hours, "hour"),
            with_suffix_if_not_zero(mins, "minute"),
            with_suffix_if_not_zero(secs, "second"),
        ];

        let vec: Vec<String> = iter.into_iter().flatten().collect();

        match vec.len() {
            0 => write!(f, "0 seconds"),
            1 => write!(f, "{}", vec[0]),
            2 => write!(f, "{} and {}", vec[0], vec[1]),
            len => {
                write!(f, "{}", vec[0])?;

                for (i, entry) in vec.into_iter().enumerate().skip(1) {
                    if i == len - 1 {
                        write!(f, " and ")?;
                    } else {
                        write!(f, ", ")?;
                    }

                    write!(f, "{entry}")?;
                }

                Ok(())
            }
        }
    }
}

#[expect(deprecated)]
impl TimeDeltaFormatter {
    #[must_use]
    pub fn seconds(secs: i64) -> Self {
        Self { secs }
    }
}
