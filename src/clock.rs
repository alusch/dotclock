use std::cell::RefCell;
use std::iter;
use std::rc::Rc;

use anyhow::{Context, Result};
use bdf::Font;
use chrono::Local;
use flipdot::{Address, Page, PageId, SerialSignBus, Sign, SignBus, SignType};
use flipdot_testing::{VirtualSign, VirtualSignBus};

use crate::options::Options;

#[derive(Debug)]
pub struct Fonts {
    pub main: Font,
    pub am_pm: Font,
    pub num_7x7: Font,
}

#[derive(Debug)]
pub struct Clock {
    options: Options,
    fonts: Fonts,
    sign: Sign,
}

impl Fonts {
    const MAIN_DATA: &'static [u8] = include_bytes!("fonts/main.bdf");
    const AM_PM_DATA: &'static [u8] = include_bytes!("fonts/am_pm.bdf");
    const NUM_7X7_DATA: &'static [u8] = include_bytes!("fonts/num_7x7.bdf");

    pub fn try_new() -> Result<Self> {
        Ok(Self {
            main: bdf::read(Self::MAIN_DATA).context("Failed to load main font")?,
            am_pm: bdf::read(Self::AM_PM_DATA).context("Failed to load AM/PM font")?,
            num_7x7: bdf::read(Self::NUM_7X7_DATA).context("Failed to load 7x7 number font")?,
        })
    }
}

impl Clock {
    pub fn try_new(options: Options) -> Result<Self> {
        let bus_rc: Rc<RefCell<dyn SignBus>>;
        if options.port.eq_ignore_ascii_case("virtual") {
            let bus = VirtualSignBus::new(iter::once(VirtualSign::new(Address(options.address))));
            bus_rc = Rc::new(RefCell::new(bus));
        } else {
            let port = serial::open(&options.port)
                .context(format!("Failed to open serial port `{}`", &options.port))?;
            let bus = SerialSignBus::try_new(port).context("Failed to create bus")?;
            bus_rc = Rc::new(RefCell::new(bus));
        }

        // TODO: Allow configuring the address and type (which will also require different ways of
        // generating the output and possibly fonts to adapt to different sizes).
        let sign = Sign::new(bus_rc.clone(), Address(options.address), SignType::Max3000Side90x7);
        sign.configure().context("Failed to configure sign")?;

        Ok(Self {
            options,
            fonts: Fonts::try_new()?,
            sign,
        })
    }

    pub fn display_time(&self) -> Result<()> {
        let now = Local::now();

        let mut page = self.sign.create_page(PageId(0));

        let date_format = if self.options.show_day_of_week {
            " %a\u{2009}%d"
        } else {
            " %b\u{2009}%d"
        };

        // Format the current time into string(s) and write to the page using BDF fonts.
        // This is all very specific to fitting nicely on the 90 x 7 sign.
        // TODO: Think about ways to clean up/generalize this.
        if self.options.use_24_hour {
            let time = now.format("%H:%M").to_string();
            let date = now.format(date_format).to_string().to_uppercase();
            let mut x_pos = 7;
            x_pos = Self::write_string(&time, &self.fonts.num_7x7, &mut page, x_pos);
            Self::write_string(&date, &self.fonts.main, &mut page, x_pos);
        } else {
            let time = now.format("%_I:%M").to_string();
            let am_pm = now.format("%p").to_string();
            let date = now.format(date_format).to_string().to_uppercase();
            let mut x_pos = 1;
            x_pos = Self::write_string(&time, &self.fonts.num_7x7, &mut page, x_pos);
            x_pos = Self::write_string(&am_pm, &self.fonts.am_pm, &mut page, x_pos);
            Self::write_string(&date, &self.fonts.main, &mut page, x_pos);
        }

        self.sign.send_pages(&[page]).context("Failed to send page")?;
        self.sign.show_loaded_page().context("Failed to show page")?;

        Ok(())
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
}
