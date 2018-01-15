#![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
#![no_builtins]
#![no_std]

extern crate compiler_builtins;

pub mod lang_items;

// The GPIO mmio base address for raspberry pi 3.
const GPIO_BASE: usize = 0x3F200000;

// Controls actuation of pull up/down to ALL GPIO pins.
#[allow(dead_code)]
const GPPUD: *mut u32 = (GPIO_BASE + 0x94) as *mut u32;

// Controls actuation of pull up/down for specific GPIO pin.
#[allow(dead_code)]
const GPPUDCLK0: *mut u32 = (GPIO_BASE + 0x98) as *mut u32;

// Function select for a specific GPIO pin, a set of 5 registers each with ten
// pins (3 bits per pin).
const GPIO_FSEL0: *mut u32 = (GPIO_BASE) as *mut u32;

// Set a GPIO output pin, set of 2 registers with one bit per pin.
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;

// Clear a GPIO output pin, set of 2 registers with one bit per pin.
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

#[inline(never)]
fn spin_sleep_ms(ms: usize) {
    for _ in 0..(ms * 6000) {
        unsafe { asm!("nop" :::: "volatile"); }
    }
}

pub struct Gpio {
    pin: usize,
}

pub enum GpioMode {
    Input,
    Output,
    AltFunction0,
    AltFunction1,
    AltFunction2,
    AltFunction3,
    AltFunction4,
    AltFunction5,
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
            GpioMode::AltFunction0 => 0b100,
            GpioMode::AltFunction1 => 0b101,
            GpioMode::AltFunction2 => 0b110,
            GpioMode::AltFunction3 => 0b111,
            GpioMode::AltFunction4 => 0b011,
            GpioMode::AltFunction5 => 0b010,

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

pub mod miniuart {
    use super::Gpio;
    use super::GpioMode;

    const AUX_BASE: usize = 0x3F215000;

    const AUX_ENABLES: *mut u32     = (AUX_BASE + 0x04) as *mut u32;
    const AUX_MU_IO_REG: *mut u32   = (AUX_BASE + 0x40) as *mut u32;
    const AUX_MU_IER_REG: *mut u32  = (AUX_BASE + 0x44) as *mut u32;
    const AUX_MU_IIR_REG: *mut u32  = (AUX_BASE + 0x48) as *mut u32;
    const AUX_MU_LCR_REG: *mut u32  = (AUX_BASE + 0x4C) as *mut u32;
    const AUX_MU_MCR_REG: *mut u32  = (AUX_BASE + 0x50) as *mut u32;
    const AUX_MU_LSR_REG: *mut u32  = (AUX_BASE + 0x54) as *mut u32;
    #[allow(dead_code)]
    const AUX_MU_MSR_REG: *mut u32  = (AUX_BASE + 0x58) as *mut u32;
    #[allow(dead_code)]
    const AUX_MU_SCRATCH: *mut u32  = (AUX_BASE + 0x5C) as *mut u32;
    const AUX_MU_CNTL_REG: *mut u32 = (AUX_BASE + 0x60) as *mut u32;
    #[allow(dead_code)]
    const AUX_MU_STAT_REG: *mut u32 = (AUX_BASE + 0x64) as *mut u32;
    const AUX_MU_BAUD_REG: *mut u32 = (AUX_BASE + 0x68) as *mut u32;

    pub unsafe fn initialize() {
        // Enable miniUART.
        AUX_ENABLES.write_volatile((AUX_ENABLES.read_volatile() & 0x7) | 0x1);

        AUX_MU_IER_REG.write_volatile(0); // Disable interrupts.
        AUX_MU_CNTL_REG.write_volatile(0); // Disable RX/TX.
        // Enable 8-bit mode. According to the docs only 0x1 is required, but
        // without setting 0x3 it doesn't work.  0x3 is what a 16550-compatible
        // UART requires to enable 8-bit, datasheet error?
        AUX_MU_LCR_REG.write_volatile(0x3); // Enable 8-bit mode.

        // "If clear the UART1_RTS line is high If set the UART1_RTS line is low"
        AUX_MU_MCR_REG.write_volatile(0);
        // Disable interrupts (not sure how this differs from AUX_MU_IER_REG).
        AUX_MU_IIR_REG.write_volatile(0);

        // The baud register is (system_clock_rate / (8 * desired_baud) - 1)
        // For 115200, this is 270.
        AUX_MU_BAUD_REG.write_volatile(270);

        Gpio::pin(14).fsel(GpioMode::AltFunction5);
        Gpio::pin(15).fsel(GpioMode::AltFunction5);

        // Enable RX/TX.
        AUX_MU_CNTL_REG.write_volatile(3);
    }

    pub unsafe fn putc(c: u8) {
        // Wait until the transmit FIFO can accept at least one byte.
        while (AUX_MU_LSR_REG.read_volatile() & 0x20) == 0 {
            continue
        }

        AUX_MU_IO_REG.write_volatile(c as u32);
    }

    pub unsafe fn getc() -> u8 {
        // Wait for the receive FIFO to hold something.
        while (AUX_MU_LSR_REG.read_volatile() & 1) == 0 {
            continue
        }

        (AUX_MU_IO_REG.read_volatile() & 0xFF) as u8
    }

    pub unsafe fn puts(value: &str) {
        let bytes: &[u8] = value.as_bytes();
        for byte in bytes {
            putc(*byte);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn kmain() {
    miniuart::initialize();

    let gpio16 = Gpio::pin(16);

    // Set GPIO Pin 16 as output.
    gpio16.fsel(GpioMode::Output);
    gpio16.set();

    spin_sleep_ms(3000);
    miniuart::puts("Hello, World!");

    // Wait for input and flash the LED on GPIO16.
    loop {
        gpio16.clear();
        miniuart::putc(miniuart::getc());
        gpio16.set();
        spin_sleep_ms(50);
    }
}
