#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(i128_type)]
#![feature(never_type)]
#![feature(unique)]
#![feature(pointer_methods)]
#![feature(naked_functions)]
#![feature(fn_must_use)]
#![feature(alloc, allocator_api, global_allocator)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate pi;
extern crate stack_vec;
extern crate fat32;

pub mod allocator;
pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;
pub mod fs;
pub mod traps;
pub mod aarch64;
pub mod process;
pub mod vm;

use pi::gpio::Gpio;
use pi::timer::spin_sleep_ms;

#[cfg(not(test))]
use allocator::Allocator;
use fs::FileSystem;
use process::GlobalScheduler;

use console::kprintln;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();

#[no_mangle]
#[cfg(not(test))]
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

    ALLOCATOR.initialize();
    FILE_SYSTEM.initialize();

    kprintln!("kmain CurrentEL: {}", unsafe { aarch64::current_el() } );

    unsafe { asm!("brk 1" :::: "volatile"); }

    kprintln!("Returned! CurrentEL: {}", unsafe { aarch64::current_el() } );
    kprintln!("Doing more things");

    unsafe { asm!("brk 2" :::: "volatile"); }

    kprintln!("CurrentEL: {}", unsafe { aarch64::current_el() } );
    kprintln!("Last print");

    loop {
        shell::shell("end>");
    }
}
