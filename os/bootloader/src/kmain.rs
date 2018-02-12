#![feature(asm, lang_items)]

extern crate xmodem;
extern crate pi;

pub mod lang_items;

use xmodem::Xmodem;
use pi::uart::MiniUart;
use pi::gpio::Gpio;

/// Start address of the binary to load and of the bootloader.
const BINARY_START_ADDR: usize = 0x80000;
const BOOTLOADER_START_ADDR: usize = 0x4000000;

/// Pointer to where the loaded binary expects to be loaded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Free space between the bootloader and the loaded binary's start address.
const MAX_BINARY_SIZE: usize = BOOTLOADER_START_ADDR - BINARY_START_ADDR;

/// Branches to the address `addr` unconditionally.
fn jump_to(addr: *mut u8) -> ! {
    unsafe {
        asm!("br $0" : : "r"(addr as usize));
        loop { asm!("nop" :::: "volatile")  }
    }
}

#[no_mangle]
pub extern "C" fn kmain() {
    {
        let mut ready_led = Gpio::new(16).into_output();
        let mut on = false;

        let mut uart = MiniUart::new();
        uart.set_read_timeout(750);

        let mut storage: &mut [u8];
        unsafe {
            storage = std::slice::from_raw_parts_mut(BINARY_START, MAX_BINARY_SIZE);
        }

        loop {
            // Toggle the ready led.
            if on {
                ready_led.clear();
            } else {
                ready_led.set();
            }
            on = !on;

            match Xmodem::receive(&mut uart, &mut storage) {
                // Receive failed, retry.
                Err(_) => continue,
                // Break out of the retry loop and load the binary.
                Ok(_) => {
                    ready_led.clear();
                    break;
                },
            }
        }
    }

    // Bootloader is loaded, jump to the start.
    jump_to(BINARY_START);
}
