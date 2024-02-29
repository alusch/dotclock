use std::cell::RefCell;
use std::iter;
use std::rc::Rc;
use std::sync::mpsc::channel;

use anyhow::{Context, Result};
use chrono::{Local, Timelike};
use flipdot::{Address, SerialSignBus, Sign, SignBus, SignType};
use flipdot_testing::{VirtualSign, VirtualSignBus};
use structopt::StructOpt;
use timer::MessageTimer;

mod clock;
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

    /// Show day of week (WED 28) after the time instead of month (FEB 28)
    #[structopt(short = "d", long = "dayofweek")]
    pub show_day_of_week: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    let options = Options::from_args();
    let address = Address(options.address);

    let bus_rc: Rc<RefCell<dyn SignBus>>;
    if options.port.eq_ignore_ascii_case("virtual") {
        let bus = VirtualSignBus::new(iter::once(VirtualSign::new(address)));
        bus_rc = Rc::new(RefCell::new(bus));
    } else {
        let port = serial::open(&options.port)
            .context(format!("Failed to open serial port `{}`", &options.port))?;
        let bus = SerialSignBus::try_new(port).context("Failed to create bus")?;
        bus_rc = Rc::new(RefCell::new(bus));
    }

    // TODO: Allow configuring the type (which will also require different ways of
    // generating the output and possibly fonts to adapt to different sizes).
    let sign = Sign::new(bus_rc.clone(), address, SignType::Max3000Side90x7);
    sign.configure().context("Failed to configure sign")?;

    let clock = Clock::try_new(sign, options.use_24_hour, options.show_day_of_week)?;
   
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
