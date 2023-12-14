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
use msp430f5529::interrupt;

fn delay(n: u16) {
    let mut i = 0;
    loop {
        asm::nop();

        i += 1;

        if i == n {
            break;
        }
    }
}

// P0 = red LED
// P6 = green LED
#[entry]
fn main() -> ! {
    let p = msp430f5529::Peripherals::take().unwrap();

    // Disable watchdog
    let wd = p.WATCHDOG_TIMER;
    wd.wdtctl().write(|w| {
        w.wdtpw().password().wdthold().set_bit()
    });

    let p12 = p.PORT_1_2;
    let p34 = p.PORT_3_4;

    // set P1.0 high and P4.7 low
    p12.p1out().modify(|_, w| w.p1out0().set_bit());
    p34.p4out().modify(|_, w| w.p4out7().clear_bit());

    // Set P1.0 and P4.7 as outputs
    p12.p1dir().modify(|_, w| w.p1dir0().set_bit());
    p34.p4dir().modify(|_, w| w.p4dir7().set_bit());

    loop {
        delay(10_000);

        // toggle outputs
        p12.p1out().modify(|r, w| w.p1out0().bit(!r.p1out0().bit()));
        p34.p4out().modify(|r, w| w.p4out7().bit(!r.p4out7().bit()));
    }
}

#[interrupt]
fn TIMER0_A0() {}
