// Work only with lcds that have hd44780 driver like lcd 1602

#![no_std]
use defmt::*;
use embassy_rp::{
    bind_interrupts,
    i2c::{ self, Async, Config, I2c, SclPin, SdaPin },
    peripherals::I2C0,
    Peri,
};
use embassy_time::Delay;
use embedded_hal_1::delay::DelayNs;
use hd44780_driver::{ bus::I2CBus, error::Error, Cursor, CursorBlink, Direction, Display, HD44780 };
use itoa::Buffer; // For integers
use ryu::Buffer as FloatBuffer; // For floats

#[derive(Debug, Clone, Copy)]
pub enum LcdError {
    ClearError,
    ResetError,
    DisplayError,
    WriteError,
    CursorError,
}

#[derive(Debug, Clone, Copy)]
pub enum CursorMoveDirection {
    Left,
    Right,
}

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

macro_rules! lcd_try {
    ($expr:expr, $error:expr) => {
        $expr.map_err(|e| {
            // Optional: Log the error or perform custom handling
            ($error, e)
        })?
    };
}
pub struct Lcd<'a> {
    driver: HD44780<I2CBus<I2c<'a, I2C0, Async>>>,
    delay: Delay,
    update_screen_time: u32,
}
impl<'d> Lcd<'d> {
    pub async fn new(
        i2c0: Peri<'d, I2C0>,
        scl: Peri<'d, impl SclPin<I2C0>>,
        sda: Peri<'d, impl SdaPin<I2C0>>,
        update_screen_time: u32
    ) -> Self {
        let config = Config::default();

        let i2c = I2c::new_async(i2c0, scl, sda, Irqs, config);

        let mut delay = Delay;

        delay.delay_ms(update_screen_time);

        let mut lcd_driver = HD44780::new_i2c(i2c, 0x27, &mut delay).unwrap();

        match Self::initialize_lcd(&mut lcd_driver, &mut delay) {
            Ok(_) => info!("LCD Initialized Successfully"),
            Err(_) => warn!("LCD Init Failed"),
        }

        Self {
            driver: lcd_driver,
            delay,
            update_screen_time: update_screen_time,
        }
    }

    fn initialize_lcd(
        lcd: &mut HD44780<I2CBus<I2c<'d, I2C0, Async>>>,
        delay: &mut Delay
    ) -> Result<(), (LcdError, Error)> {
        lcd_try!(lcd.clear(delay), LcdError::ClearError);
        lcd_try!(lcd.reset(delay), LcdError::ResetError);
        lcd_try!(lcd.set_display(Display::On, delay), LcdError::DisplayError);
        lcd_try!(lcd.set_cursor_pos(0, delay), LcdError::DisplayError);
        lcd_try!(lcd.set_cursor_visibility(Cursor::Invisible, delay), LcdError::CursorError);
        lcd_try!(lcd.set_cursor_blink(CursorBlink::Off, delay), LcdError::CursorError);
        Ok(())
    }

    pub async fn display_text(
        &mut self,
        text: &str,
        clear_display: bool
    ) -> Result<(), (LcdError, Error)> {
        self.delay.delay_ms(self.update_screen_time);
        if clear_display {
            self.clear_display().await?;
        }
        self.driver.write_str(text, &mut self.delay).map_err(|e| (LcdError::WriteError, e))
    }

    pub async fn display_byte(
        &mut self,
        byte: u8,
        clear_display: bool
    ) -> Result<(), (LcdError, Error)> {
        self.delay.delay_ms(self.update_screen_time);
        if clear_display {
            self.clear_display().await?;
        }
        self.driver.write_byte(byte, &mut self.delay).map_err(|e| (LcdError::WriteError, e))
    }

    pub async fn display_bytes(
        &mut self,
        bytes: &[u8],
        clear_display: bool
    ) -> Result<(), (LcdError, Error)> {
        self.delay.delay_ms(self.update_screen_time);
        if clear_display {
            self.clear_display().await?;
        }
        self.driver.write_bytes(bytes, &mut self.delay).map_err(|e| (LcdError::WriteError, e))
    }

    pub async fn display_char(
        &mut self,
        char: char,
        clear_display: bool
    ) -> Result<(), (LcdError, Error)> {
        self.delay.delay_ms(self.update_screen_time);
        if clear_display {
            self.clear_display().await?;
        }
        self.driver.write_char(char, &mut self.delay).map_err(|e| (LcdError::WriteError, e))
    }

    pub async fn display_int<T: itoa::Integer>(
        &mut self,
        num: T,
        clear_display: bool
    ) -> Result<(), (LcdError, Error)> {
        let mut buffer = Buffer::new();
        if clear_display {
            self.clear_display().await?;
        }
        self.driver
            .write_str(buffer.format(num), &mut self.delay)
            .map_err(|e| (LcdError::WriteError, e))
    }

    pub async fn display_float<T: ryu::Float>(
        &mut self,
        num: T,
        clear_display: bool
    ) -> Result<(), (LcdError, Error)> {
        let mut buffer = FloatBuffer::new();
        if clear_display {
            self.clear_display().await?;
        }
        self.driver
            .write_str(buffer.format(num), &mut self.delay)
            .map_err(|e| (LcdError::WriteError, e))
    }

    pub async fn clear_display(&mut self) -> Result<(), (LcdError, Error)> {
        self.driver.clear(&mut self.delay).map_err(|e| (LcdError::ClearError, e))
    }

    pub async fn reset_display(&mut self) -> Result<(), (LcdError, Error)> {
        self.delay.delay_ms(self.update_screen_time);
        self.driver.reset(&mut self.delay).map_err(|e| (LcdError::ResetError, e))
    }

    pub async fn set_display_mode(
        &mut self,
        display_mode: Display
    ) -> Result<(), (LcdError, Error)> {
        self.delay.delay_ms(self.update_screen_time);
        self.driver
            .set_display(display_mode, &mut self.delay)
            .map_err(|e| (LcdError::DisplayError, e))
    }

    pub async fn set_cursor_visibility(&mut self, visible: bool) -> Result<(), (LcdError, Error)> {
        self.delay.delay_ms(self.update_screen_time);
        let visibillity: Cursor;
        if visible {
            visibillity = Cursor::Visible;
        } else {
            visibillity = Cursor::Invisible;
        }
        self.driver
            .set_cursor_visibility(visibillity, &mut self.delay)
            .map_err(|e| (LcdError::CursorError, e))
    }

    pub async fn set_cursor_blink(&mut self, blink: bool) -> Result<(), (LcdError, Error)> {
        self.delay.delay_ms(self.update_screen_time);
        let cursor_blink: CursorBlink;
        if blink {
            cursor_blink = CursorBlink::On;
        } else {
            cursor_blink = CursorBlink::Off;
        }
        self.driver
            .set_cursor_blink(cursor_blink, &mut self.delay)
            .map_err(|e| (LcdError::CursorError, e))
    }

    pub async fn move_cursor_direction(
        &mut self,
        move_direction: CursorMoveDirection
    ) -> Result<(), (LcdError, Error)> {
        match move_direction {
            CursorMoveDirection::Left => {
                self.driver
                    .shift_cursor(Direction::Left, &mut self.delay)
                    .map_err(|e| (LcdError::CursorError, e))
            }
            CursorMoveDirection::Right => {
                self.driver
                    .shift_cursor(Direction::Right, &mut self.delay)
                    .map_err(|e| (LcdError::CursorError, e))
            }
        }
    }

    pub async fn set_autoscroll(&mut self, enable: bool) -> Result<(), (LcdError, Error)> {
        self.driver.set_autoscroll(enable, &mut self.delay).map_err(|e| (LcdError::CursorError, e))
    }

    pub async fn set_cursor_pos(&mut self, position: u8) -> Result<(), (LcdError, Error)> {
        self.delay.delay_ms(self.update_screen_time);
        self.driver
            .set_cursor_pos(position, &mut self.delay)
            .map_err(|e| (LcdError::CursorError, e))
    }
}
