#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(ptr_internals)]
#![feature(pointer_methods)]

extern crate pi;
extern crate stack_vec;

pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;

use pi::timer::spin_sleep_ms;


const GPIO_BASE: usize = 0x3F000000 + 0x200000;

const GPIO_FSEL0: *mut u32 = (GPIO_BASE) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

pub struct Gpio {
    pin: usize,
}

pub enum GpioMode {
    Input,
    Output
}

impl Gpio {
    pub fn pin(pin: usize) -> Gpio {
        if pin <= 53 {
            Gpio { pin }
        } else {
            panic!("Invalid pin number");
        }
    }

    pub fn fsel(&self, mode: GpioMode) {
        let fsel_register: usize = self.pin / 10;
        let fsel_offset: usize = (self.pin - fsel_register * 10) * 3;

        let flags: u32 = match mode {
            GpioMode::Input => 0b000,
            GpioMode::Output => 0b001,
        };

        unsafe {
            let reg: *mut u32 = GPIO_FSEL0.offset(fsel_register as isize);
            let value: u32 = reg.read_volatile() & !(0b111 << fsel_offset);
            reg.write_volatile(value | (flags << fsel_offset));
        }
    }

    pub fn set(&self) {
        let gpio_register: usize = self.pin / 32;
        let gpio_offset: usize = self.pin - gpio_register * 32;

        unsafe {
            let reg: *mut u32 = GPIO_SET0.offset(gpio_register as isize);
            reg.write_volatile(1 << gpio_offset);
        }
    }

    pub fn clear(&self) {
        let gpio_register: usize = self.pin / 32;
        let gpio_offset: usize = self.pin - gpio_register * 32;

        unsafe {
            let reg: *mut u32 = GPIO_CLR0.offset(gpio_register as isize);
            reg.write_volatile(1 << gpio_offset);
        }
    }
}


#[no_mangle]
pub extern "C" fn kmain() {
    let gpio16 = Gpio::pin(16);

    // Set GPIO Pin 16 as output.
    gpio16.fsel(GpioMode::Output);

    // Continuously set and clear GPIO 16.
    loop {
        gpio16.set();
        spin_sleep_ms(10);
        gpio16.clear();
        spin_sleep_ms(100);
    }
}
