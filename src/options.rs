use structopt::StructOpt;

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
