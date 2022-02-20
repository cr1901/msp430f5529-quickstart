//! Basic "hello world" blink demo for the [MSP-EXP430G2](http://www.ti.com/tool/MSP-EXP430G2)
//! development kit using a software delay loop- in Rust!
//!
//! Although unnecessary for running the demo, this example also shows the syntax for declaring
//! an interrupt.
//!
//! ---

#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

extern crate panic_msp430;

use msp430::asm;
use msp430_rt::entry;
#[allow(unused)]
use msp430f5529::interrupt;

use core::convert::Infallible;
use hmac_sha256::HMAC;
use hmac_sha512::HMAC as HMAC512;
use ufmt::uwrite;
use ufmt_write::uWrite;

struct UartWriter(msp430f5529::USCI_A1_UART_MODE);

impl UartWriter {
    fn new(uart: msp430f5529::USCI_A1_UART_MODE) -> Self {
        UartWriter(uart)
    }
}

impl uWrite for UartWriter {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Infallible> {
        s.as_bytes().iter().for_each(|b| {
            while self.0.uca1ifg.read().uctxifg().bit_is_clear() {}

            self.0.uca1txbuf.write(|w| unsafe { w.bits(*b) });
        });

        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let p = msp430f5529::Peripherals::take().unwrap();

    // Disable watchdog
    let wd = p.WATCHDOG_TIMER;
    wd.wdtctl.write(|w| {
        unsafe { w.bits(0x5A00) } // password
            .wdthold()
            .set_bit()
    });

    // MCLK and SCLK are 32768*32 = 1048576
    // Enable backchannel UART
    let port4 = p.PORT_3_4;
    let uart = p.USCI_A1_UART_MODE;
    port4.p4dir.write(|w| w.p4dir5().set_bit());

    // Set I/O to use UART
    port4
        .p4sel
        .write(|w| w.p4sel4().set_bit().p4sel5().set_bit());

    uart.uca1ctl1
        .write(|w| w.ucssel().ucssel_2().ucswrst().set_bit());

    // UCA1BRW = 3;
    uart.uca1br0.write(|w| unsafe { w.bits(3) });

    uart.uca1br1.write(|w| unsafe { w.bits(0) });

    // UCA1MCTL |= (UCBRF_6 | UCBRS_1 | UCOS16);
    // Overampling mode, 19200 baud.
    // Copied from manual.
    uart.uca1mctl
        .write(|w| w.ucbrf().ucbrf_6().ucbrs().ucbrs_1().ucos16().set_bit());

    // Enable UART.
    uart.uca1ctl1.modify(|_, w| w.ucswrst().clear_bit());

    // Garbled (after button reset)... why?
    // uart.uca1txbuf.write(|w| {
    //     unsafe { w.bits('a' as u8) }
    // });

    let mut writer = UartWriter::new(uart);

    // Divide by 16 (8, 2) to get 65536 Hz timer.
    // Count up to 1 second for benchmark.
    let timer = p.TIMER_0_A5;
    // Divide by 2.
    timer.ta0ex0.write(|w| w.taidex().taidex_1());

    // Count from 0 to 65535, use SCLK, divide by 8.
    timer.ta0ctl.write(|w| w.tassel().tassel_2().id().id_3());

    // Do benchmark- reset timer val, start timer (continuous).
    timer.ta0r.write(|w| unsafe { w.bits(0) });
    timer.ta0ctl.modify(|_, w| w.mc().mc_2());

    let h = HMAC::mac(&[], &[0u8; 32]);
    assert_eq!(
        &h[..],
        &[
            182, 19, 103, 154, 8, 20, 217, 236, 119, 47, 149, 215, 120, 195, 95, 197, 255, 22, 151,
            196, 147, 113, 86, 83, 198, 199, 18, 20, 66, 146, 197, 173
        ]
    );

    // Stop the timer.
    timer.ta0ctl.modify(|_, w| w.mc().mc_0());

    let elapsed = timer.ta0r.read().bits();
    uwrite!(writer, "hmac_sha256: {}/65536 seconds\r\n", elapsed).unwrap();

    // Next benchmark
    timer.ta0r.write(|w| unsafe { w.bits(0) });
    timer.ta0ctl.modify(|_, w| w.mc().mc_2());

    let h = HMAC512::mac(&[], &[0u8; 32]);
    assert_eq!(
        &h[..],
        &[
            185, 54, 206, 232, 108, 159, 135, 170, 93, 60, 111, 46, 132, 203, 90, 66, 57, 165, 254,
            80, 72, 10, 110, 198, 107, 112, 171, 91, 31, 74, 198, 115, 12, 108, 81, 84, 33, 179,
            39, 236, 29, 105, 64, 46, 83, 223, 180, 154, 215, 56, 30, 176, 103, 179, 56, 253, 123,
            12, 178, 34, 71, 34, 93, 71
        ]
    );

    timer.ta0ctl.modify(|_, w| w.mc().mc_0());

    let elapsed = timer.ta0r.read().bits();
    uwrite!(writer, "hmac_sha512: {}/65536 seconds\r\n", elapsed).unwrap();

    // We are done!
    uwrite!(writer, "bench.rs Okay!\r\n").unwrap();
    loop {
        asm::nop();
    }
}
