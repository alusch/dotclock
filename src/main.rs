use std::sync::mpsc::channel;

use anyhow::{Context, Result};
use chrono::{Local, Timelike};
use flipdot_graphics::{Address, FlipdotDisplay, SignBusType, SignType};
use structopt::StructOpt;
use timer::MessageTimer;

mod clock;
mod fonts;
use clock::Clock;

#[derive(StructOpt, Debug)]
#[structopt(about, author)]
pub struct Options {
    /// Serial port to use to connect to a real sign. Pass "virtual" to use a virtual sign
    /// for testing (set environment variable RUST_LOG=flipdot=info to view output).
    pub port: String,

    /// Address of the sign to show the clock on
    #[structopt(short = "a", long = "address")]
    pub address: u16,

    /// Uses 24-hour time formatting (14:30 instead of 2:30 PM)
    #[structopt(short = "t", long = "24hour")]
    pub use_24_hour: bool,

    /// Shows day of week (WED 28) after the time instead of month (FEB 28)
    #[structopt(short = "d", long = "dayofweek")]
    pub show_day_of_week: bool,

    /// Shows the current time and exits (suitable for use with cron, etc.)
    #[structopt(short = "o", long = "oneshot")]
    pub one_shot: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    let options = Options::from_args();

    // TODO: Allow configuring the sign type (which will also require different ways of
    // generating the output and possibly fonts to adapt to different sizes).
    let display = FlipdotDisplay::try_new(
        SignBusType::from(&options.port),
        Address(options.address),
        SignType::Max3000Side90x7,
    )?;
    let mut clock = Clock::new(display, options.use_24_hour, options.show_day_of_week);

    if options.one_shot {
        clock.display_time()
    } else {
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
}
