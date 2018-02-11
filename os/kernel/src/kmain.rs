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

extern crate pi;
extern crate stack_vec;

pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;

use pi::timer::spin_sleep_ms;
use pi::gpio::Gpio;

#[no_mangle]
pub extern "C" fn kmain() {
    let mut gpio16 = Gpio::new(16).into_output();

    // Continuously set and clear GPIO 16.
    loop {
        gpio16.set();
        spin_sleep_ms(100);
        gpio16.clear();
        spin_sleep_ms(100);
    }
}
