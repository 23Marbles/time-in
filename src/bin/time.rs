use chrono::Local;
use clap::Parser;
use timein::{StaticStrError, arg::Args, time::TimeValue};

fn main() -> color_eyre::Result {
    let args = Args::parse();

    let Args { cmd } = args;

    let now = Local::now();

    match cmd
        .destructure()
        .ok_or(StaticStrError("Failed getting datetime / duration input"))?
    {
        (TimeValue::Duration(naive_duration), fmt, show_true_time_passage) => {
            let then = naive_duration
                .datetime_checked_add(now.with_timezone(&chrono::Utc))
                .ok_or(StaticStrError("Failed adding duration to datetime"))?;

            match (naive_duration.is_zero(), naive_duration.is_neg()) {
                (true, _) => print!("The time is {}", fmt.format_datetime(&then)),
                (false, true) => print!("The time was {}", fmt.format_datetime(&then)),
                (false, false) => print!("The time will be {}", fmt.format_datetime(&then)),
            }

            if show_true_time_passage {
                let passed = then.signed_duration_since(now).abs();

                if naive_duration.is_neg() {
                    println!(".\n{} has passed", fmt.format_duration(passed));
                } else {
                    println!(".\n{} will have passed", fmt.format_duration(passed));
                }
            } else {
                println!();
            }
        }
        (TimeValue::DateTime(naive_date_time), fmt, _) => {
            let time_utc = if let Some(tz) = fmt.timezone {
                naive_date_time
                    .and_local_timezone(tz)
                    .single()
                    .expect("ambiguous or invalid local time")
                    .to_utc()
            } else {
                naive_date_time.and_utc()
            };

            let now_utc = now.to_utc();

            // time_utc is in the past
            if now_utc > time_utc {
                let delta = now_utc.signed_duration_since(time_utc);

                println!(
                    "{} was {} ago",
                    fmt.format_datetime(&time_utc),
                    fmt.format_duration(delta),
                );
            } else if now_utc < time_utc {
                let delta = time_utc.signed_duration_since(now_utc);

                println!(
                    "It is {} until {}",
                    fmt.format_duration(delta),
                    fmt.format_datetime(&time_utc),
                );
            } else {
            }
        }
    }

    Ok(())
}
