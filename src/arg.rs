use chrono::NaiveDateTime;
use chrono_tz::Tz;

use crate::{
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

    #[command(flatten)]
    format: Format<chrono_tz::Tz>,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    Now {
        #[command(flatten)]
        options: CommonOptions,
    },

    /// Finds the datetime an amount of time in the future
    In {
        #[arg(num_args = 1..)]
        duration: Vec<TimeUnitDur>,

        #[arg(long)]
        show_duration: bool,

        #[command(flatten)]
        options: CommonOptions,
    },

    /// Finds the datetime an amount of time ago
    Past {
        #[arg(num_args = 1..)]
        duration: Vec<TimeUnitDur>,

        #[arg(long)]
        show_duration: bool,

        #[command(flatten)]
        options: CommonOptions,
    },

    /// Finds the duration to a date time.
    #[command(visible_alias = "since", visible_alias = "till")]
    To {
        date: DateParser,
        time: TimeParser,

        #[command(flatten)]
        options: CommonOptions,
    },
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
            Command::To {
                date,
                time,
                options,
            } => (
                TimeValue::DateTime(NaiveDateTime::new(date.0, time.0)),
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
            Command::To { date, time, .. } => {
                TimeValue::DateTime(NaiveDateTime::new(date.0, time.0))
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
