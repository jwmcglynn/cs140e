use pi::interrupt::Interrupt;

use traps::TrapFrame;
use pi::timer::tick_in;
use process::{State, TICK};
use console::kprintln;
use SCHEDULER;


pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    if interrupt == Interrupt::Timer1 {
        kprintln!(".");
        tick_in(TICK);
        SCHEDULER.switch(State::Ready, tf);
    }

    // Unmask IRQ.
    tf.spsr &= !(1 << 7);
}
