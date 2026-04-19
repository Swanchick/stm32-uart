#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate stm32l4xx_hal as hal;
use defmt_rtt as _;
use panic_probe as _;

use cortex_m::Peripherals as CpuPeripherals;
use hal::{
    adc::ADC, delay::Delay, flash::FlashExt, prelude::*, pwr::PwrExt, rcc::RccExt, serial::Serial,
};
use rt::entry;

use core::fmt::Write;

use stm32l4xx_hal::pac::Peripherals as McuPeripherals;

#[entry]
fn main() -> ! {
    let cpu = CpuPeripherals::take().unwrap();
    let mcu = McuPeripherals::take().unwrap();

    let mut flash = mcu.FLASH.constrain();
    let mut rcc = mcu.RCC.constrain();
    let mut pwr = mcu.PWR.constrain(&mut rcc.apb1r1);

    let clocks = rcc
        .cfgr
        .sysclk(80.MHz())
        .pclk1(80.MHz())
        .pclk2(80.MHz())
        .freeze(&mut flash.acr, &mut pwr);

    let mut gpioa = mcu.GPIOA.split(&mut rcc.ahb2);
    let mut analog = gpioa.pa0.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);

    let tx = gpioa
        .pa2
        .into_alternate::<7>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

    let rx = gpioa
        .pa3
        .into_alternate::<7>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);

    let mut delay = Delay::new(cpu.SYST, clocks);

    let mut adc = ADC::new(
        mcu.ADC1,
        mcu.ADC_COMMON,
        &mut rcc.ahb2,
        &mut rcc.ccipr,
        &mut delay,
    );

    let serial = Serial::usart2(mcu.USART2, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb1r1);
    let (mut tx, _) = serial.split();

    loop {
        let value = adc.read(&mut analog);

        if let Ok(value) = value {
            let voltage = (value as f32) * 3.3 / 4095.0;

            writeln!(tx, "Potentiometer value: {}\r", voltage).ok();

            delay.delay_ms(500u32);
        }
    }
}
