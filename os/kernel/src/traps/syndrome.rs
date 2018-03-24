#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8)
}

impl From<u32> for Fault {
    fn from(val: u32) -> Fault {
        use self::Fault::*;

        match (val & 0b111111) as u8 {
            0b000000...0b000011 => AddressSize,
            0b000100...0b000111 => Translation,
            0b001001...0b001011 => AccessFlag,
            0b001101...0b001111 => Permission,
            0b100001 => Alignment,
            0b110000 => TlbConflict,
            other => Other(other),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    McrMrc,
    McrrMrrc,
    LdcStc,
    SimdFp,
    Vmrs,
    Mrrc,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort {
        kind: Fault,
        level: u8,
    },
    PCAlignmentFault,
    DataAbort {
        kind: Fault,
        level: u8
    },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32)
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Syndrome {
        use self::Syndrome::*;

        let exception_class = esr >> 26;
        match exception_class {
            0b000000 => Unknown,
            0b000000 => WfiWfe,
            0b000011 => McrMrc,
            0b000100 => McrrMrrc,
            0b000101 => McrMrc,
            0b000110 => LdcStc,
            0b000111 => SimdFp,
            0b001000 => Vmrs,
            0b001100 => Mrrc,
            0b001110 => IllegalExecutionState,
            // AArch32
            0b010001 => Svc((esr & 0xFFFF) as u16),
            0b010010 => Hvc((esr & 0xFFFF) as u16),
            0b010011 => Smc((esr & 0xFFFF) as u16),
            // AArch64
            0b010101 => Svc((esr & 0xFFFF) as u16),
            0b010110 => Hvc((esr & 0xFFFF) as u16),
            0b010111 => Smc((esr & 0xFFFF) as u16),
            0b011000 => MsrMrsSystem,
            // From lower exception level.
            0b100000 => InstructionAbort { kind: Fault::from(esr), level: 0 },
            // From same level.
            0b100001 => InstructionAbort { kind: Fault::from(esr), level: 1 },
            0b100010 => PCAlignmentFault,
            // From lower exception level.
            0b100100 => DataAbort { kind: Fault::from(esr), level: 0 },
            // From same level.
            0b100101 => DataAbort { kind: Fault::from(esr), level: 1 },
            0b100110 => SpAlignmentFault,
            0b101000 => TrappedFpu,  // AArch32
            0b101100 => TrappedFpu,  // AArch64
            0b101111 => SError,
            0b110000 => Breakpoint,  // From lower exception level.
            0b110001 => Breakpoint,  // From same level.
            0b110010 => Step,  // From lower exception level.
            0b110011 => Step,  // From same level.
            0b110100 => Watchpoint,  // From lower exception level.
            0b110101 => Watchpoint,  // From same level.
            0b111000 => Brk((esr & 0xFFFF) as u16),  // AArch32
            0b111100 => Brk((esr & 0xFFFF) as u16),  // AArch64
            // Includes implementation defined and vector catch.
            other => Other(other)
        }
    }
}
