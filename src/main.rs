use std::sync::mpsc::channel;

use anyhow::{Context, Result};
use chrono::{Local, Timelike};
use structopt::StructOpt;
use timer::MessageTimer;

mod clock;
mod options;

use clock::Clock;
use options::Options;

fn main() -> Result<()> {
    env_logger::init();

    let clock = Clock::try_new(Options::from_args())?;
   
    // Capture the current time, but set the seconds and nanoseconds to 0.
    // (This is safe to unwrap since 0 is a valid value). This becomes our initial target time.
    let now = Local::now();
    let even_minute = now.with_second(0).unwrap().with_nanosecond(0).unwrap();

    // Set up recurring callbacks every minute, on the minute. Since our initial time is
    // in the past by construction, this will always run immediately to show the current time.
    // Then, regular updates occur on the minute, giving us proper clock-like operation.
    // TODO: This won't be totally accurate since it doesn't account for the time to
    // send the messages or for the sign to display them. Could add some smarts to
    // try to tune that. For a basic clock though, that's probably overkill.
    let (tx, rx) = channel();
    let timer = MessageTimer::new(tx);
    let _guard = timer.schedule(even_minute, Some(chrono::Duration::minutes(1)), ());

    loop {
        rx.recv().context("Channel failure")?;
        clock.display_time()?;
    }
}
