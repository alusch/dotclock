use anyhow::Result;
use chrono::Local;
use eg_bdf::BdfTextStyle;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use flipdot_graphics::FlipdotDisplay;

use crate::fonts::{FONT_AM_PM, FONT_MAIN, FONT_NUM_7X7};

#[derive(Debug)]
pub struct Fonts {
    pub main: BdfTextStyle<'static, BinaryColor>,
    pub am_pm: BdfTextStyle<'static, BinaryColor>,
    pub num_7x7: BdfTextStyle<'static, BinaryColor>,
}

#[derive(Debug)]
pub struct Clock {
    display: FlipdotDisplay,
    fonts: Fonts,
    use_24_hour: bool,
    show_day_of_week: bool,
}

impl Default for Fonts {
    fn default() -> Self {
        Self {
            main: BdfTextStyle::new(&FONT_MAIN, BinaryColor::On),
            am_pm: BdfTextStyle::new(&FONT_AM_PM, BinaryColor::On),
            num_7x7: BdfTextStyle::new(&FONT_NUM_7X7, BinaryColor::On),
        }
    }
}

impl Clock {
    pub fn new(display: FlipdotDisplay, use_24_hour: bool, show_day_of_week: bool) -> Self {
        Self {
            display,
            fonts: Fonts::default(),
            use_24_hour,
            show_day_of_week,
        }
    }

    pub fn display_time(&mut self) -> Result<()> {
        let now = Local::now();

        self.display.clear(BinaryColor::Off)?;

        let date_format = if self.show_day_of_week {
            " %a\u{2009}%d"
        } else {
            " %b\u{2009}%d"
        };

        // Format the current time into string(s) and write to the page using BDF fonts.
        // This is all very specific to fitting nicely on the 90 x 7 sign.
        // TODO: Think about ways to clean up/generalize this.
        if self.use_24_hour {
            let time = now.format("%H:%M").to_string();
            let date = now.format(date_format).to_string().to_uppercase();
            let next = Point::new(7, 0);
            let next = Text::with_baseline(&time, next, self.fonts.num_7x7, Baseline::Top)
                .draw(&mut self.display)?;
            Text::new(&date, next, self.fonts.main).draw(&mut self.display)?;
        } else {
            let time = now.format("%_I:%M").to_string();
            let am_pm = now.format("%p").to_string();
            let date = now.format(date_format).to_string().to_uppercase();
            let next = Point::new(1, 0);
            let next = Text::with_baseline(&time, next, self.fonts.num_7x7, Baseline::Top)
                .draw(&mut self.display)?;
            let next = Text::new(&am_pm, next, self.fonts.am_pm).draw(&mut self.display)?;
            Text::new(&date, next, self.fonts.main).draw(&mut self.display)?;
        }

        self.display.flush()?;

        Ok(())
    }
}
