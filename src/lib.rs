use std::{
    env,
    io::{Write, stdout},
};

use chrono::{Datelike, Local, TimeDelta};

use crate::{
    parse::{ArgParseError, split_snippet},
    time::{DAY_IN_SECS, HOUR_IN_SECS, MIN_IN_SECS, TimeKind, WEEK_IN_SECS},
};

pub mod arg;
pub mod format;
pub mod parse;
pub mod time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
#[error("{0}")]
pub struct StaticStrError(pub &'static str);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
#[error("{0}")]
pub struct StringError(pub String);

#[expect(dead_code)]
#[deprecated]
fn old_main() -> color_eyre::Result<()> {
    let args = env::args();

    let mut delta_secs = 0;

    let mut show_date = false;
    let mut show_duration = false;
    let mut show_weekday = false;

    for arg in args.skip(1) {
        match arg.as_str() {
            "--use-date" | "--date" | "-da" => {
                show_date = true;
                continue;
            }
            "--show-weekday" | "--weekday" | "-wd" => {
                show_weekday = true;
                continue;
            }
            "--show-duration" | "--duration" | "--dur" | "-du" => {
                show_duration = true;
                continue;
            }
            _ => {}
        }

        let secs = match split_snippet(&arg) {
            Ok((k, n)) => match k {
                TimeKind::Secs => n,
                TimeKind::Mins => {
                    n.checked_mul(MIN_IN_SECS)
                        .ok_or(ArgParseError::DigitTooLarge(format!(
                            "{}",
                            i128::from(n) * i128::from(MIN_IN_SECS)
                        )))?
                }
                TimeKind::Hours => {
                    n.checked_mul(HOUR_IN_SECS)
                        .ok_or(ArgParseError::DigitTooLarge(format!(
                            "{}",
                            i128::from(n) * i128::from(HOUR_IN_SECS)
                        )))?
                }
                TimeKind::Weeks => {
                    n.checked_mul(WEEK_IN_SECS)
                        .ok_or(ArgParseError::DigitTooLarge(format!(
                            "{}",
                            i128::from(n) * i128::from(WEEK_IN_SECS)
                        )))?
                }
                TimeKind::Days => {
                    n.checked_mul(DAY_IN_SECS)
                        .ok_or(ArgParseError::DigitTooLarge(format!(
                            "{}",
                            i128::from(n) * i128::from(DAY_IN_SECS)
                        )))?
                }
                _ => unimplemented!("changeable time units"),
            },
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        delta_secs += i128::from(secs);
    }

    let secs = i64::try_from(delta_secs)
        .map_err(|_| ArgParseError::DigitTooLarge(delta_secs.to_string()))?;

    let now = Local::now();

    let then = now + TimeDelta::seconds(secs);

    let mut stdout = stdout().lock();

    write!(
        &mut stdout,
        "The time will be {} ",
        then.format("%-l:%M %p")
    )?;

    if show_weekday {
        write!(&mut stdout, "on {}", then.format("%A"))?;
    }

    if show_date {
        write!(&mut stdout, "on {}", then.format("%x"))?;
    } else {
        write!(&mut stdout, "{}", {
            let now_days = now.num_days_from_ce();
            let then_days = then.num_days_from_ce();
            match then_days - now_days {
                0 => "today".to_string(),
                1 => "tommorow".to_string(),
                n => format!("{n} days from now"),
            }
        })?;
    }

    #[expect(deprecated)]
    if show_duration {
        write!(
            &mut stdout,
            " in {}",
            time::TimeDeltaFormatter::seconds(secs)
        )?;
    }

    writeln!(&mut stdout)?;

    Ok(())
}
