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
    let mut loading_leds = [
        Gpio::new(5).into_output(),
        Gpio::new(6).into_output(),
        Gpio::new(13).into_output(),
        Gpio::new(19).into_output(),
        Gpio::new(26).into_output()];

    for ref mut led in loading_leds.iter_mut() {
        led.set();
        spin_sleep_ms(100);
    }

    spin_sleep_ms(2000);

    for ref mut led in loading_leds.iter_mut() {
        led.clear();
        spin_sleep_ms(100);
    }

    shell::shell("> ");
}
