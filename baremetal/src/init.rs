mod boot {
    use core::arch::global_asm;
    // First thing that runs thanks to the linker script.
    // Assumed we're running on core 1, so we set the stack pointer
    // accordingly; this should change per core of course.
    global_asm!(
        "
        .section .text._start
        .globl _start
        _start:
            # Set the stack pointer for EL1.
            ldr x0, = _stack_start_1
            msr sp_el1, x0

            # Configure hypervisor configuration register (HCR)
            # to avoid trapping exceptions at EL2.
            mrs     x0, hcr_el2
            orr     x0, x0, #(1 << 31)
            bic     x0, x0, #(1 << 5)
            bic     x0, x0, #(1 << 4)
            bic     x0, x0, #(1 << 3)
            msr     hcr_el2, x0

            # Disable SIMD and FPU instruction trapping at EL2
            msr cptr_el2, xzr

            # Reset sctlr_el1 to zeros to avoid faults
            msr     sctlr_el1, xzr

            # Reset all TLB caches
            tlbi alle1
            dsb sy
            isb

            # Set DAIF to 1111 to avoid interrupts, set M[4:0] to 0b00101 also.
            mov     x0, 0b1111000101
            msr     spsr_el2, x0
            adr     x0, _el1_setup
            msr     elr_el2, x0

            isb
            eret

        _el1_setup:

            # Enable FPU and SIMD, since rust likes to use instructions that require this.
            mov     x0, #(3 << 20)
            msr     cpacr_el1, x0

            bl _rust_main
            "
    );
}

pub mod tables {

    use core::arch::asm;

    const TABLE2_SIZE: usize = 512;

    pub fn setup_mmu() {
        #[repr(C, align(4096))]
        struct TranslationTable([u64; TABLE2_SIZE]);

        const fn create_translation_table2() -> TranslationTable {
            let mut out = [0; TABLE2_SIZE];
            let mut i = 0;

            while i < TABLE2_SIZE {
                let mut val: u64 = 0;

                if i < 128 {
                    // Leave as 0; this is linux memory and we don't want to map it.
                } else {
                    if i < 504 {
                        // Normal memory.1
                        val |= 0b0 << 54; // Set XN[1] to 0
                        val |= 0b0 << 53; // Set XN[0] to 0 (executable from both EL1 and EL0)
                        val |= 0b11 << 8; // Set SH (shareability -> inner shareable)
                        val |= 0b000 << 2; // Set AttrIndex to 0 so we read ATTR0 from MAIR_EL1.
                    } else {
                        // Peripheral memory.
                        val |= 0b1 << 54; // Set XN[1] to 1
                        val |= 0b0 << 53; // Set XN[0] to 0 (non-executable from both EL1 and EL0)
                        val |= 0b10 << 8; // Set SH (shareability -> outer shareable)
                        val |= 0b001 << 2; // Set AttrIndex to 1 so we read ATTR1 from MAIR_EL1.
                    }

                    val |= 0b1 << 10; // Set AF (access flag to 1 since not accessed)
                    val |= 0b00 << 6; // S2AP to 11 (access permission to read/write)
                    val |= 0b0 << 5; // Set NS (non-secure = 0 means we are in secure address map)

                    val |= (i as u64) << 21; // Set translation destination!

                    val |= 0b01; // Set block descriptor to 01, meaning

                    out[i] = val;
                }

                i += 1;
            }

            TranslationTable(out)
        }

        static TRANSLATION_TABLE_LVL2: TranslationTable = create_translation_table2();

        const TCR_EL1: u64 = {
            let mut out: u64 = 0;

            out |= 0b101 << 32; // IPS (intermediate physical address size) is 48.
            out |= 0b10 << 30; // TG1; set granule size to 4K (for 2MB lvl 2).
            out |= 0b11 << 28; // SH1 (shareability) set to inner-shareable.
            out |= 0b01 << 26; // ORGN1 set outer cacheability to normal outer write-back write-allocate.
            out |= 0b01 << 24; // IRGN1 set inner cacheability to the same
            out |= 0b1 << 23; // EPD1, DISABLE TABLE WALKING on TLB miss; we only want lower addresses to be translated.
            out |= 0b0 << 22; // A1; use TTBR0_EL1 ASID for ASID (good because we have no TTBR1_EL1).
            out |= 34 << 16; // T1SZ; set to 34 so that only 30 bits are used for addressing (only bottom 30 bits are relevant).
            out |= 0b00 << 14; // TG0, set to 4K granule size.
            out |= 0b11 << 12; // SH0; Shareability (same as above)
            out |= 0b01 << 10; // ORGN0 outer cacheability (see above)
            out |= 0b01 << 8; // IRGN0 inner cacheability (see above)
            out |= 0b0 << 7; // EPD0; enable table walks with TTBR0_EL1
            out |= 34; // T0SZ; (same as above) set for 30 bit max address.

            out
        };

        const MAIR_EL1: u64 = {
            let mut out: u64 = 0;
            out |= 0b00000000 << 8; // Set ATTR1 to Device-nGnRnE.
            out |= 0b11111111; // Set ATTR0 to Normal Inner Write-Back Non-Transient Allocate memory.
            out
        };

        unsafe {
            asm!(
                "
                    # disable exceptions so we are not interrupted.
                    msr DAIFSet, 0b1111 


                    # Set relevant registers
                    msr TCR_EL1, {tcr_el1}
                    msr MAIR_EL1, {mair_el1}
                    msr TTBR0_EL1, {table_base_addr}
                    msr TTBR1_EL1, {table_base_addr}

                    # Invalidate TLB cache
                    tlbi vmalle1

                    # Enable MMU!
                    mrs {tmp}, sctlr_el1
                    orr {tmp}, {tmp}, 0x1
                    orr {tmp}, {tmp}, (0x1 << 12)
                    msr sctlr_el1, {tmp}

                    dsb ish
                    isb
                ",
                tcr_el1 =  in(reg) TCR_EL1,
                mair_el1 = in(reg) MAIR_EL1,
                table_base_addr = in(reg) &TRANSLATION_TABLE_LVL2 as *const TranslationTable as u64,
                tmp = out(reg) _,
            );
        }
    }
}
