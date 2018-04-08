use common::IO_BASE;
use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile, Reserved};

const INT_BASE: usize = IO_BASE + 0xB000 + 0x200;

#[derive(Copy, Clone, PartialEq)]
pub enum Interrupt {
    Timer1 = 1,
    Timer3 = 3,
    Usb = 9,
    Gpio0 = 49,
    Gpio1 = 50,
    Gpio2 = 51,
    Gpio3 = 52,
    Uart = 57,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    pending_basic: Reserved<u32>,
    pending: [ReadVolatile<u32>; 2],
    fiq_control: Reserved<u32>,
    enable: [Volatile<u32>; 2],
    enable_basic: Reserved<u32>,
    disable: [Volatile<u32>; 2],
    disable_basic: Reserved<u32>,
}

/// An interrupt controller. Used to enable and disable interrupts as well as to
/// check if an interrupt is pending.
pub struct Controller {
    registers: &'static mut Registers
}

impl Controller {
    /// Returns a new handle to the interrupt controller.
    pub fn new() -> Controller {
        Controller {
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    /// Enables the interrupt `int`.
    pub fn enable(&mut self, int: Interrupt) {
        let index = int as u64;
        if index < 32 {
            self.registers.enable[0].or_mask(1 << index);
        } else {
            self.registers.enable[1].or_mask(1 << (index - 32));
        }
    }

    /// Disables the interrupt `int`.
    pub fn disable(&mut self, int: Interrupt) {
        let index = int as u64;
        if index < 32 {
            self.registers.disable[0].or_mask(1 << index);
        } else {
            self.registers.disable[1].or_mask(1 << (index - 32));
        }
    }

    /// Returns `true` if `int` is pending. Otherwise, returns `false`.
    pub fn is_pending(&self, int: Interrupt) -> bool {
        let index = int as u64;
        if index < 32 {
            self.registers.pending[0].has_mask(1 << index)
        } else {
            self.registers.pending[1].has_mask(1 << (index - 32))
        }
    }
}
