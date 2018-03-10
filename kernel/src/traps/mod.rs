mod irq;
mod trap_frame;
mod syndrome;

use pi::interrupt::{Controller, Interrupt};

pub use self::trap_frame::TrapFrame;

use console::kprintln;
use self::syndrome::Syndrome;
use self::irq::handle_irq;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

#[no_mangle]
pub extern fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    unimplemented!("handle_exception")
}
