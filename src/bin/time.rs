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
                .ok_or(StaticStrError("Failed finding adding duration to datetime"))?;

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
        (TimeValue::DateTime(_naive_date_time), _fmt, _) => todo!(),
    }

    Ok(())
}
