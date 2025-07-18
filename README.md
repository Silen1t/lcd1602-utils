# lcd1602-utils

🚀 Async utility crate for controlling **I2C LCD1602 (HD44780-based)** displays on **Raspberry Pi Pico W** using the [Embassy](https://embassy.dev) async embedded framework.

> ⚠️ **Works only with LCDs that use the HD44780 controller over an I²C backpack** (e.g., PCF8574).

---

## ✨ Features

- 🔌 I²C-based async LCD1602 driver using `embassy-rp` and `hd44780-driver`
- 🕹 Cursor control (position, direction, blink, visibility)
- 📟 Write characters, strings, integers, floats
- 🔄 Reset / clear / autoscroll
- 🕒 Configurable screen update delay
- 🔧 Error handling enum for clear diagnostics

---

## How to use it
```rs
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_time::{ Duration, Timer };
use lcd_driver::Lcd;
use ::{ defmt_rtt as _, panic_probe as _ };

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Init LCD
    let mut lcd = Lcd::new(p.I2C0, p.PIN_17, p.PIN_16, 20).await;
    lcd.clear_display().await.unwrap();
    lcd.display_text("Hello World", false).await.unwrap();

}

```
