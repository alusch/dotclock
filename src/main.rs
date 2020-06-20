use std::cell::RefCell;
use std::iter;
use std::rc::Rc;
use std::sync::mpsc::channel;

use anyhow::{Context, Result};
use bdf::Font;
use chrono::{Local, Timelike};
use flipdot::{Address, Page, PageId, SerialSignBus, Sign, SignBus, SignType};
use flipdot_testing::{VirtualSign, VirtualSignBus};
use structopt::StructOpt;
use timer::MessageTimer;

#[derive(StructOpt, Debug)]
#[structopt(about, author)]
struct Opts {
    /// Serial port to use to connect to a real sign. Pass "virtual" to use a virtual sign
    /// for testing (set environment variable RUST_LOG=flipdot=info to view output).
    port: String,

    /// Uses 24-hour time formatting (14:30 instead of 2:30 PM)
    #[structopt(short = "t", long = "24hour")]
    use_24_hour: bool,
}

fn main() -> Result<()> {
    env_logger::init();

    let opts = Opts::from_args();
    if opts.port.eq_ignore_ascii_case("virtual") {
        let bus = VirtualSignBus::new(iter::once(VirtualSign::new(Address(3))));
        show_clock(Rc::new(RefCell::new(bus)), opts.use_24_hour)?;
    } else {
        let port = serial::open(&opts.port)
            .context(format!("Failed to open serial port `{}`", &opts.port))?;
        let bus = SerialSignBus::try_new(port).context("Failed to create bus")?;
        show_clock(Rc::new(RefCell::new(bus)), opts.use_24_hour)?;
    }

    Ok(())
}

fn show_clock(bus: Rc<RefCell<dyn SignBus>>, use_24_hour: bool) -> Result<()> {
    // Load up resources and parse BDF fonts.
    const MAIN_FONT_DATA: &[u8] = include_bytes!("fonts/main.bdf");
    const AM_PM_FONT_DATA: &[u8] = include_bytes!("fonts/am_pm.bdf");
    const NUM_FONT_7X7_DATA: &[u8] = include_bytes!("fonts/num_7x7.bdf");

    let main_font = bdf::read(MAIN_FONT_DATA).context("Failed to load main font")?;
    let am_pm_font = bdf::read(AM_PM_FONT_DATA).context("Failed to load AM/PM font")?;
    let num_font_7x7 = bdf::read(NUM_FONT_7X7_DATA).context("Failed to load 7x7 number font")?;

    // Create the sign.
    // TODO: Allow configuring the address and type (which will also require different ways of
    // generating the output and possibly fonts to adapt to different sizes).
    let sign = Sign::new(bus.clone(), Address(3), SignType::Max3000Side90x7);
    sign.configure().context("Failed to configure sign")?;

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
        let now = Local::now();

        let mut page = sign.create_page(PageId(0));

        // Format the current time into string(s) and write to the page using BDF fonts.
        // This is all very specific to fitting nicely on the 90 x 7 sign.
        // TODO: Think about ways to clean up/generalize this.
        if use_24_hour {
            let time = now.format("%H:%M").to_string();
            let date = now.format(" %b\u{2009}%d").to_string().to_uppercase();
            let mut x_pos = 7;
            x_pos = write_string(&time, &num_font_7x7, &mut page, x_pos);
            write_string(&date, &main_font, &mut page, x_pos);
        } else {
            let time = now.format("%_I:%M").to_string();
            let am_pm = now.format("%p").to_string();
            let date = now.format(" %b\u{2009}%d").to_string().to_uppercase();
            let mut x_pos = 1;
            x_pos = write_string(&time, &num_font_7x7, &mut page, x_pos);
            x_pos = write_string(&am_pm, &am_pm_font, &mut page, x_pos);
            write_string(&date, &main_font, &mut page, x_pos);
        }

        sign.send_pages(&[page]).context("Failed to send page")?;
        sign.show_loaded_page().context("Failed to show page")?;
    }
}

/// Writes the given `string` using the provided `font` into a `page`, starting at
/// the column given by `x_start`. Returns the column after the last one written to,
/// to facilitate using multiple calls to append different strings.
///
/// Glyphs not present in `font` are ignored. Panics if it exceeds the dimensions of the page
/// (font is too tall or string is too long). Everything is top-aligned on the assumption
/// that the font will match the height of the page.
fn write_string(string: &str, font: &Font, page: &mut Page<'_>, x_start: u32) -> u32 {
    let mut x_offset = x_start;
    for codepoint in string.chars() {
        if let Some(glyph) = font.glyphs().get(&codepoint) {
            for y in 0..glyph.height() {
                for x in 0..glyph.width() {
                    page.set_pixel(x_offset + x, y, glyph.get(x, y));
                }
            }
            x_offset += glyph.width();
        }
    }
    x_offset
}
