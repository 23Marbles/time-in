use std::{convert::Infallible, error::Error, fmt::Display, str::FromStr};

use chrono::{DateTime, Datelike, Local, TimeDelta, TimeZone, Utc};
use num_integer::Integer;

#[cfg(feature = "show_weeks")]
use crate::time::WEEK_IN_SECS;
use crate::time::{DAY_IN_SECS, HOUR_IN_SECS, MIN_IN_SECS};

#[derive(Debug, Clone, Default)]
pub enum DateFormatter {
    #[default]
    Auto,
    DaysFromNow,
    Date,
    Both,
    Formatted(String),
}

impl Display for DateFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DateFormatter::Auto => "auto",
                DateFormatter::DaysFromNow => "days-from-now",
                DateFormatter::Date => "date",
                DateFormatter::Both => "both",
                DateFormatter::Formatted(s) => s,
            }
        )
    }
}

impl std::str::FromStr for DateFormatter {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "days-from-now" | "dfn" => Ok(Self::DaysFromNow),
            "date" => Ok(Self::Date),
            "both" => Ok(Self::Both),
            format => Ok(Self::Formatted(format.to_owned())),
        }
    }
}

impl DateFormatter {
    pub fn format_date<Tz>(&self, date: &DateTime<Tz>, show_weekday: bool) -> String
    where
        Tz: TimeZone + Display,
        Tz::Offset: Display,
    {
        fn format_date<Tz>(date: &DateTime<Tz>, show_weekday: bool) -> String
        where
            Tz: TimeZone + Display,
            Tz::Offset: Display,
        {
            if show_weekday {
                date.format("%A, %-e %B %Y").to_string()
            } else {
                date.format("on %-e %B %Y").to_string()
            }
        }

        fn format_day_diff<Tz>(date: &DateTime<Tz>, show_weekday: bool) -> String
        where
            Tz: TimeZone + Display,
            Tz::Offset: Display,
        {
            let day_diff = {
                let today = Local::now().num_days_from_ce();

                let date_day = date.num_days_from_ce();

                date_day - today
            };

            let (weeks, days) = day_diff.div_rem(&7);

            let raw = match (
                days.unsigned_abs(),
                weeks.unsigned_abs(),
                day_diff.is_negative(),
            ) {
                // Special names
                (0, 0, _) => "today".to_string(),
                (1, 0, false) => "tommorow".to_string(),
                (1, 0, true) => "yesterday".to_string(),

                // Within a week
                (n, 0, false) => format!("in {n} days time"),
                (n, 0, true) => format!("{n} days ago"),

                // Within two weeks
                (d, 1, false) => format!("in a week and {d} days"),
                (d, 1, true) => format!("a week and {d} days ago"),

                // On an exact week diff
                (0, w, false) => format!("in {w} weeks"),
                (0, w, true) => format!("{w} weeks ago"),

                // Multiple weeks and singluar day
                (1, w, false) => format!("in {w} weeks and a day"),
                (1, w, true) => format!("{w} weeks and a day ago"),

                // Multiple weeks and days
                (d, w, false) => format!("in {w} weeks and {d} days"),
                (d, w, true) => format!("{w} weeks and {d} days ago"),
            };

            if show_weekday {
                format!("{}, {raw}", date.format("%A"))
            } else {
                raw
            }
        }

        match self {
            DateFormatter::Auto => {
                let day_diff = {
                    let today = Local::now().num_days_from_ce();

                    let date_day = date.num_days_from_ce();

                    date_day - today
                };

                if day_diff.abs() <= 7 {
                    format_day_diff(date, show_weekday)
                } else {
                    format_date(date, show_weekday)
                }
            }
            DateFormatter::DaysFromNow => format_day_diff(date, show_weekday),
            DateFormatter::Date => format_date(date, show_weekday),
            DateFormatter::Both => {
                format!(
                    "on {}, {}",
                    format_date(date, show_weekday),
                    format_day_diff(date, false)
                )
            }
            DateFormatter::Formatted(format_str) => date.format(format_str).to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum TimeFormatter {
    Utc,
    #[default]
    Local,
    Formatted(String),
}

impl Display for TimeFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TimeFormatter::Utc => "utc",
                TimeFormatter::Local => "local",
                TimeFormatter::Formatted(s) => &s,
            }
        )
    }
}

impl std::str::FromStr for TimeFormatter {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "utc" => Ok(Self::Utc),
            "local" => Ok(Self::Local),
            format => Ok(Self::Formatted(format.to_owned())),
        }
    }
}

impl TimeFormatter {
    #[must_use]
    pub fn format_time<Tz>(&self, time: &DateTime<Utc>, show_timezone: Option<&Tz>) -> String
    where
        Tz::Offset: Display,
        Tz: TimeZone + Display,
    {
        match (self, show_timezone) {
            (TimeFormatter::Utc, None) => time.format("%-H:%M:%S"),
            (TimeFormatter::Local, None) => time.with_timezone(&Local).format("%-H:%M:%S"),
            (TimeFormatter::Utc | TimeFormatter::Local, Some(tz)) => {
                time.with_timezone(tz).format("%-H:%M:%S %Z")
            }
            (TimeFormatter::Formatted(format_str), None) => time.format(format_str),
            (TimeFormatter::Formatted(format_str), Some(tz)) => {
                time.with_timezone(tz).format(format_str)
            }
        }
        .to_string()
    }
}

#[derive(Debug, Clone, clap::Args)]
pub struct Format<Tz>
where
    Tz::Offset: Display,
    Tz: TimeZone + Display + Send + Sync + FromStr + 'static,
    <Tz as FromStr>::Err: Send + Sync + Error,
{
    #[arg(long, default_value_t = TimeFormatter::default())]
    pub time_formatter: TimeFormatter,

    #[arg(long)]
    pub show_timezone: bool,

    #[arg(skip)]
    pub timezone: Option<Tz>,

    #[arg(long, default_value_t = DateFormatter::default())]
    pub date_formatter: DateFormatter,

    #[arg(long)]
    pub show_weekday: bool,

    #[arg(long)]
    pub show_date_first: bool,
}

impl<Tz> Format<Tz>
where
    Tz::Offset: Display,
    Tz: TimeZone + Display + Send + Sync + FromStr + 'static,
    <Tz as FromStr>::Err: Send + Sync + Error,
{
    #[must_use]
    pub fn with_timezone(mut self, tz: Option<Tz>) -> Self {
        self.timezone = tz;
        self.show_timezone = true;
        self
    }

    pub fn format_datetime(&self, datetime: &DateTime<Utc>) -> String
    where
        Tz::Offset: Display,
        Tz: TimeZone + Display,
    {
        let date = self.date_formatter.format_date(datetime, self.show_weekday);

        let time = self.time_formatter.format_time(
            datetime,
            self.show_timezone
                .then_some(self.timezone.as_ref())
                .flatten(),
        );

        if self.show_date_first {
            format!("{date} {time}")
        } else {
            format!("{time} {date}")
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn format_duration(&self, duration: TimeDelta) -> String {
        fn with_suffix_if_not_zero(num: i64, suffix: &str) -> Option<String> {
            if num == 0 {
                return None;
            }

            Some(format!("{num} {suffix}{}", if num == 1 { "" } else { "s" }))
        }

        let rem = duration.num_seconds();

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
            0 => "0 seconds".to_string(),
            1 => vec
                .into_iter()
                .nth(0)
                .expect("The length must be 1, so indexing at 0 is allowed"),
            2 => format!("{} and {}", vec[0], vec[1]),
            len => {
                let mut s = String::new();

                s.push_str(&vec[0]);

                for (i, entry) in vec.into_iter().enumerate().skip(1) {
                    if i == len - 1 {
                        s.push_str(" and ");
                    } else {
                        s.push_str(", ");
                    }

                    s.push_str(&entry);
                }

                s
            }
        }
    }
}
