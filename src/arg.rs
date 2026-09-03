use std::str::FromStr;

use chrono::{Local, NaiveDateTime};
use chrono_tz::Tz;

use crate::{
    StringError,
    format::Format,
    parse::{DateParser, TimeParser},
    time::{NaiveDuration, TimeUnitDur, TimeValue},
};

#[derive(Debug, clap::Parser)]
#[command(author, version)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Clone, clap::Args)]
pub struct CommonOptions {
    #[arg(long, short, visible_alias = "tz")]
    /// Specifies a timezone to work in
    timezone: Option<chrono_tz::Tz>,

    /// Specifies the formatting to output with
    #[command(flatten)]
    format: Format<chrono_tz::Tz>,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Displays the time now
    Now {
        #[command(flatten)]
        options: CommonOptions,
    },

    /// Finds the datetime an amount of time in the future
    In {
        /// The duration, is a list of consecutive snippets parsed as value-key pairs.
        /// These are parsed as <number><Ident> where Ident can be any of the following values:
        /// ["years", "months", "weeks", "days", "hours", "minutes", "seconds"],
        /// or their shorthands or singluar form
        #[arg(num_args = 1.., verbatim_doc_comment)]
        duration: Vec<TimeUnitDur>,

        /// Whether to show the duration in the output message
        #[arg(long)]
        show_duration: bool,

        #[command(flatten)]
        options: CommonOptions,
    },

    /// Finds the datetime an amount of time ago
    Past {
        /// The duration, is a list of consecutive snippets parsed as value-key pairs.
        /// These are parsed as <number><Ident> where Ident can be any of the following values:
        /// ["years", "months", "weeks", "days", "hours", "minutes", "seconds"],
        /// or their shorthands or singluar form
        #[arg(num_args = 1.., verbatim_doc_comment)]
        duration: Vec<TimeUnitDur>,

        /// Whether to show the duration in the output message
        #[arg(long)]
        show_duration: bool,

        #[command(flatten)]
        options: CommonOptions,
    },

    /// Finds the duration to a date time.
    #[command(visible_alias = "since", visible_alias = "until", alias = "till")]
    To {
        /// This can be either a date or time, or both consecutively.
        /// Dates take the format: yyyy-mm-dd and
        /// time takes the format: hh:mm[:ss[.fract]] where anything in "[]" is optional
        #[arg(num_args = 1..=2, verbatim_doc_comment)]
        datetime: Vec<TimeOrDateParser>,

        #[command(flatten)]
        options: CommonOptions,
    },
}

#[derive(Debug, Clone)]
pub enum TimeOrDateParser {
    Date(DateParser),
    Time(TimeParser),
}

impl FromStr for TimeOrDateParser {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(date) = DateParser::from_str(s) {
            Ok(Self::Date(date))
        } else if let Ok(time) = TimeParser::from_str(s) {
            Ok(Self::Time(time))
        } else {
            Err(StringError(format!(
                "Failed to parse `{s}` as either date or time"
            )))
        }
    }
}

pub fn datetime_from_time_or_dates(
    mut time_or_dates: impl Iterator<Item = TimeOrDateParser>,
) -> Result<NaiveDateTime, StringError> {
    let first_val = time_or_dates.next().ok_or(StringError(
        "Need at least one time or date, got none".to_string(),
    ))?;

    let (date, time) = match (first_val, time_or_dates.next()) {
        (TimeOrDateParser::Date(date), None) => (date.0, Local::now().naive_local().time()),
        (TimeOrDateParser::Time(time), None) => (Local::now().naive_local().date(), time.0),

        (TimeOrDateParser::Date(date), Some(TimeOrDateParser::Time(time)))
        | (TimeOrDateParser::Time(time), Some(TimeOrDateParser::Date(date))) => (date.0, time.0),

        (TimeOrDateParser::Date(_), Some(TimeOrDateParser::Date(_))) => {
            return Err(StringError("Two dates found".to_string()));
        }
        (TimeOrDateParser::Time(_), Some(TimeOrDateParser::Time(_))) => {
            return Err(StringError("Two times found".to_string()));
        }
    };

    Ok(NaiveDateTime::new(date, time))
}

impl Command {
    #[must_use]
    pub fn destructure(self) -> Option<(TimeValue, Format<chrono_tz::Tz>, bool)> {
        Some(match self {
            Command::Now { options } => (
                TimeValue::Duration(NaiveDuration::from_time_unit_durs(Vec::new().iter())?),
                options.format.with_timezone(options.timezone),
                false,
            ),
            Command::In {
                duration,
                options,
                show_duration,
            } => (
                TimeValue::Duration(NaiveDuration::from_time_unit_durs(duration.iter())?),
                options.format.with_timezone(options.timezone),
                show_duration,
            ),
            Command::Past {
                duration,
                options,
                show_duration,
            } => (
                TimeValue::Duration(
                    NaiveDuration::from_time_unit_durs(duration.iter())?.flip_sign(),
                ),
                options.format.with_timezone(options.timezone),
                show_duration,
            ),
            Command::To { datetime, options } => (
                TimeValue::DateTime(datetime_from_time_or_dates(datetime.into_iter()).ok()?),
                options.format.with_timezone(options.timezone),
                false,
            ),
        })
    }

    #[must_use]
    pub fn get_input_normalized(&self) -> Option<TimeValue> {
        Some(match self {
            Command::Now { .. } => {
                TimeValue::Duration(NaiveDuration::from_time_unit_durs(Vec::new().iter())?)
            }
            Command::In { duration, .. } => {
                TimeValue::Duration(NaiveDuration::from_time_unit_durs(duration.iter())?)
            }
            Command::Past { duration, .. } => TimeValue::Duration(
                NaiveDuration::from_time_unit_durs(duration.iter())?.flip_sign(),
            ),
            Command::To { datetime, .. } => {
                TimeValue::DateTime(datetime_from_time_or_dates(datetime.iter().cloned()).ok()?)
            }
        })
    }

    #[must_use]
    pub fn timezone(&self) -> Option<Tz> {
        match self {
            Command::Now {
                options: CommonOptions { timezone, .. },
                ..
            }
            | Command::In {
                options: CommonOptions { timezone, .. },
                ..
            }
            | Command::Past {
                options: CommonOptions { timezone, .. },
                ..
            }
            | Command::To {
                options: CommonOptions { timezone, .. },
                ..
            } => *timezone,
        }
    }

    #[must_use]
    pub fn format(&self) -> &Format<chrono_tz::Tz> {
        match self {
            Command::Now {
                options: CommonOptions { format, .. },
                ..
            }
            | Command::In {
                options: CommonOptions { format, .. },
                ..
            }
            | Command::Past {
                options: CommonOptions { format, .. },
                ..
            }
            | Command::To {
                options: CommonOptions { format, .. },
                ..
            } => format,
        }
    }
}
