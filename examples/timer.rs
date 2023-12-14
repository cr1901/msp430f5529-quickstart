//! Sharing data between a main thread and an interrupt handler safely.
//!
//! This example uses the [libcore](core)-provided [RefCell](core::cell::RefCell) to safely share
//! access to msp430 peripherals between a main thread and interrupt.
//!
//! As with [timer-unsafe] and [timer-oncecell], this example uses the `TIMER0_A1` interrupt to
//! blink LEDs on the [MSP-EXP430G2](http://www.ti.com/tool/MSP-EXP430G2) development kit.
//!
//! ---

#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

extern crate panic_msp430;

use core::cell::RefCell;
use msp430::{interrupt as mspint, interrupt::CriticalSection};
use msp430_rt::entry;
use msp430f5529::{interrupt, Peripherals};

static PERIPHERALS: mspint::Mutex<RefCell<Option<Peripherals>>> =
    mspint::Mutex::new(RefCell::new(None));

fn init(cs: CriticalSection) {
    let p = Peripherals::take().unwrap();

    let wdt = &p.WATCHDOG_TIMER;
    wdt.wdtctl().write(|w| {
        w.wdtpw().password().wdthold().set_bit()
    });

    let port_1_2 = &p.PORT_1_2;
    let port_3_4 = &p.PORT_3_4;
    // set P1.0 high and P4.7 low
    port_1_2.p1out().modify(|_, w| w.p1out0().set_bit());
    port_3_4.p4out().modify(|_, w| w.p4out7().clear_bit());

    // Set P1.0 and P4.7 as outputs
    port_1_2.p1dir().modify(|_, w| w.p1dir0().set_bit());
    port_3_4.p4dir().modify(|_, w| w.p4dir7().set_bit());

    let clock = &p.UCS;
    // Use REFO clock to source ACLK- 32768 Hz, nominally.
    clock.ucsctl4().modify(|_, w| w.sela().sela_2());
    // Divide it by 4, down to ~8192 Hz.
    clock.ucsctl5().modify(|_, w| w.diva().diva_2());

    let timer = &p.TIMER0_A5;
    timer.taccr0().write(|w| w.taccr0().bits(1200) );
    // Use ACLK as source, count uo to TACCR0 value (arbitrary, not too fast)
    timer
        .tactl()
        .modify(|_, w| w.tassel().tassel_1().mc().mc_1());
    timer.tacctl1().modify(|_, w| w.ccie().set_bit());
    // Fire interrupt halfway to 1200 each time.
    timer.taccr1().write(|w| w.taccr1().bits(600));

    *PERIPHERALS.borrow(cs).borrow_mut() = Some(p);
}

#[entry(interrupt_enable(pre_interrupt = init))]
fn main() -> ! {
    loop {}
}

#[interrupt]
fn TIMER0_A1(cs: CriticalSection) {
    let p_ref = PERIPHERALS.borrow(cs).borrow();
    let p = p_ref.as_ref().unwrap();

    let timer = &p.TIMER0_A5;
    timer.tacctl1().modify(|_, w| w.ccifg().clear_bit());

    let port_1_2 = &p.PORT_1_2;
    port_1_2
        .p1out()
        .modify(|r, w| w.p1out0().bit(!r.p1out0().bit()));

    let port_3_4 = &p.PORT_3_4;
    port_3_4.p4out().modify(|r, w| w.p4out7().bit(!r.p4out7().bit()));
}
