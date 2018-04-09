#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct TrapFrame {
    pub elr: u64,
    pub spsr: u64,
    pub sp: u64,
    pub tpidr: u64,
    pub q: [u128; 32],
    pub x1_to_x29: [u64; 29],
    __r0: u64,
    pub x30: u64,
    pub x0: u64,
}
