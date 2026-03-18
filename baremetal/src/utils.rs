use core::arch::asm;

pub enum Register {
    CurrentEL,
    HCR_EL2,
}

pub unsafe fn read_register(reg: Register) -> u64 {
    let mut out: u64;
    unsafe {
        match reg {
            Register::CurrentEL => {
                asm!(
                    "mrs {out}, CurrentEL",
                    out = out(reg) out
                );
            }
            Register::HCR_EL2 => {
                asm!(
                    "mrs {out}, HCR_EL2",
                    out = out(reg) out
                );
            }
        }
    }

    out
}
