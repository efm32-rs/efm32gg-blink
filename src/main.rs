#![no_main]
#![no_std]

use core::panic::PanicInfo;

use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::{entry, exception};
use efm32gg_pac::efm32gg990::gpio;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

static mut COUNT: u32 = 0;

#[allow(non_snake_case)]
#[exception]
fn SysTick() {
    unsafe {
        COUNT = COUNT.wrapping_add(1);
    }
}

fn delay(ticks_ms: u32) {
    unsafe {
        let end = COUNT.wrapping_add(ticks_ms);

        while end > COUNT {
            cortex_m::asm::wfi();
        }
    }
}

fn init_systick(cortex_periph: &mut cortex_m::Peripherals) {
    let syst = &mut cortex_periph.SYST;
    const DEFAULT_HZ: u32 = 14_000_000u32;

    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(DEFAULT_HZ / 1_000u32);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();
}

#[entry]
fn main() -> ! {
    let mut cortex_periph = cortex_m::Peripherals::take().unwrap();
    // PE2 = LED0
    // PE3 = LED1
    init_systick(&mut cortex_periph);
    let efm32 = efm32gg_pac::efm32gg990::Peripherals::take().unwrap();
    // enable GPIO clock
    efm32.CMU.hfperclken0.write(|w_reg| w_reg.gpio().set_bit());
    efm32.GPIO.pe_model.write(|w_reg| {
        w_reg
            .mode3()
            .variant(gpio::pe_model::MODE3_A::PUSHPULL)
            .mode2()
            .variant(gpio::pe_model::MODE2_A::PUSHPULL)
    });
    efm32
        .GPIO
        .pe_doutset
        .write(|w_reg| w_reg.doutset().variant(1 << 2));
    efm32
        .GPIO
        .pe_doutclr
        .write(|w_reg| w_reg.doutclr().variant(1 << 3));

    let mut is_set = true;

    loop {
        delay(250);

        if !is_set {
            efm32
                .GPIO
                .pe_doutclr
                .write(|w_reg| w_reg.doutclr().variant(1 << 3));
            efm32
                .GPIO
                .pe_doutset
                .write(|w_reg| w_reg.doutset().variant(1 << 2));
        } else {
            efm32
                .GPIO
                .pe_doutclr
                .write(|w_reg| w_reg.doutclr().variant(1 << 2));
            efm32
                .GPIO
                .pe_doutset
                .write(|w_reg| w_reg.doutset().variant(1 << 3));
        }

        is_set = !is_set
    }
}
