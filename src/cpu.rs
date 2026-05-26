//! ARM7TDMI CPU implementation for the GBA
//!
//! The GBA uses an ARM7TDMI processor supporting:
//! - ARM mode (32-bit instructions)
//! - Thumb mode (16-bit instructions)
//! - Multiple processor modes (User, IRQ, FIQ, Supervisor, Abort, Undefined, System)

bitflags::bitflags! {
    /// CPU Status Register flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CpsrFlags: u32 {
        const N = 1 << 31; // Negative
        const Z = 1 << 30; // Zero
        const C = 1 << 29; // Carry
        const V = 1 << 28; // Overflow
        const IRQ = 1 << 7; // IRQ disable
        const FIQ = 1 << 6; // FIQ disable
        const THUMB = 1 << 5; // Thumb state bit
    }
}

/// Processor operating modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    User = 0b10000,
    Fiq = 0b10001,
    Irq = 0b10010,
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
}

impl Mode {
    pub fn from_bits(bits: u32) -> Self {
        match bits & 0x1F {
            0b10000 => Mode::User,
            0b10001 => Mode::Fiq,
            0b10010 => Mode::Irq,
            0b10011 => Mode::Supervisor,
            0b10111 => Mode::Abort,
            0b11011 => Mode::Undefined,
            0b11111 => Mode::System,
            _ => Mode::System,
        }
    }
}

/// ARM7TDMI CPU
pub struct Cpu {
    // General purpose registers
    // R0-R7 are unprivileged
    // R8-R12 have banked versions for FIQ
    // R13 (SP), R14 (LR), R15 (PC/SPSR) are banked for most modes
    r: [u32; 16],

    // Banked registers for different modes
    // We'll store these as separate arrays and swap based on mode
    banked_r8_fiq: u32,
    banked_r9_fiq: u32,
    banked_r10_fiq: u32,
    banked_r11_fiq: u32,
    banked_r12_fiq: u32,

    banked_sp: [u32; 6], // For FIQ, IRQ, Supervisor, Abort, Undefined, System
    banked_lr: [u32; 6],
    banked_spsr: [u32; 6], // Saved PSR for privileged modes

    // Current program status register
    cpsr: u32,

    // Pipeline state
    pipeline: [u32; 3],    // Prefetched instructions
    pipeline_pc: [u32; 3], // PC values for each prefetched instruction
    pipeline_loaded: bool,
    pc_written: bool,
    halted: bool,

    // Trace buffer for debugging
    #[cfg(debug_assertions)]
    trace_buf: std::collections::VecDeque<(u32, u32, [u32; 16], u32)>,
    #[cfg(debug_assertions)]
    trace_enabled: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            r: [0; 16],
            banked_r8_fiq: 0,
            banked_r9_fiq: 0,
            banked_r10_fiq: 0,
            banked_r11_fiq: 0,
            banked_r12_fiq: 0,
            banked_sp: [0; 6],
            banked_lr: [0; 6],
            banked_spsr: [0; 6],
            cpsr: 0x0000001F, // System mode
            pipeline: [0; 3],
            pipeline_pc: [0; 3],
            pipeline_loaded: false,
            pc_written: false,
            halted: false,

            #[cfg(debug_assertions)]
            trace_buf: std::collections::VecDeque::with_capacity(60),
            #[cfg(debug_assertions)]
            trace_enabled: false,
        }
    }

    pub fn reset(&mut self) {
        // On reset, CPU starts in System mode
        self.r = [0; 16];
        self.banked_r8_fiq = 0;
        self.banked_r9_fiq = 0;
        self.banked_r10_fiq = 0;
        self.banked_r11_fiq = 0;
        self.banked_r12_fiq = 0;
        self.banked_sp = [0; 6];
        self.banked_lr = [0; 6];
        self.banked_spsr = [0; 6];
        self.cpsr = 0x000000DF; // System mode, IRQ/FIQ disabled, ARM mode
        self.r[13] = 0x0300_7F00; // SP (stack pointer) - points to IWRAM
        self.r[14] = 0x0800_0004; // LR (link register)
        self.r[15] = 0x0800_0000; // PC (program counter) - ROM entry point
        self.pipeline = [0; 3];
        self.pipeline_pc = [0; 3];
        self.pipeline_loaded = false;
        self.pc_written = false;
        self.halted = false;
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    #[cfg(debug_assertions)]
    pub fn enable_trace(&mut self) {
        self.trace_enabled = true;
        self.trace_buf.clear();
    }

    #[cfg(debug_assertions)]
    pub fn get_trace(&self) -> &std::collections::VecDeque<(u32, u32, [u32; 16], u32)> {
        &self.trace_buf
    }

    #[cfg(debug_assertions)]
    fn trace_record(&mut self, pc: u32, opcode: u32) {
        if !self.trace_enabled {
            return;
        }
        if self.trace_buf.len() >= 60 {
            self.trace_buf.pop_front();
        }
        self.trace_buf.push_back((pc, opcode, self.r, self.cpsr));
    }

    #[cfg(not(debug_assertions))]
    fn trace_record(&mut self, _pc: u32, _opcode: u32) {}

    pub fn set_halted(&mut self) {
        self.halted = true;
    }

    pub fn clear_halted(&mut self) {
        self.halted = false;
    }

    pub fn take_interrupt(&mut self, mem: &mut super::Memory) {
        let old_cpsr = self.cpsr;
        let ret_addr = self.get_pc();

        mem.set_bios_read_return(0xE25EF004);

        self.set_mode(Mode::Irq);

        self.set_lr(ret_addr);

        self.banked_spsr[2] = old_cpsr;

        self.cpsr |= 0x80;

        let isr_addr = mem.read_word(0x0300_7FFC);
        self.set_pc(isr_addr);
        self.pipeline_loaded = false;
    }

    // Register access
    pub fn get_reg(&self, n: usize) -> u32 {
        self.r[n]
    }

    pub fn set_reg(&mut self, n: usize, val: u32) {
        self.r[n] = val;
    }

    pub fn get_sp(&self) -> u32 {
        self.r[13]
    }

    pub fn set_sp(&mut self, val: u32) {
        self.r[13] = val;
    }

    pub fn get_lr(&self) -> u32 {
        self.r[14]
    }

    pub fn set_lr(&mut self, val: u32) {
        self.r[14] = val;
    }

    pub fn get_pc(&self) -> u32 {
        self.r[15]
    }

    pub fn get_cpsr(&self) -> u32 {
        self.cpsr
    }

    pub fn set_pc(&mut self, val: u32) {
        self.r[15] = val & 0xFFFFFFFC;
        self.pipeline_loaded = false;
        self.pc_written = true;
    }

    pub fn get_pipeline_pc(&self, idx: usize) -> u32 {
        self.pipeline_pc[idx]
    }

    pub fn get_pipeline(&self, idx: usize) -> u32 {
        self.pipeline[idx]
    }

    pub fn is_pipeline_loaded(&self) -> bool {
        self.pipeline_loaded
    }

    // Mode access
    pub fn get_mode(&self) -> Mode {
        Mode::from_bits(self.cpsr)
    }

    pub fn set_mode(&mut self, mode: Mode) {
        let current = self.get_mode();
        if current == mode {
            return;
        }

        // Save current banked registers
        let idx = self.mode_index(current);
        if idx < 6 {
            self.banked_sp[idx] = self.r[13];
            self.banked_lr[idx] = self.r[14];
            self.banked_spsr[idx] = self.get_spsr();
        }

        // Load new banked registers
        let idx = self.mode_index(mode);
        self.cpsr = (self.cpsr & !0x1F) | (mode as u32);

        if idx < 6 {
            self.r[13] = self.banked_sp[idx];
            self.r[14] = self.banked_lr[idx];
            // SPSR is loaded when needed
        }

        // Handle FIQ banking (R8-R12)
        if mode == Mode::Fiq {
            self.banked_r8_fiq = self.r[8];
            self.banked_r9_fiq = self.r[9];
            self.banked_r10_fiq = self.r[10];
            self.banked_r11_fiq = self.r[11];
            self.banked_r12_fiq = self.r[12];
        } else if current == Mode::Fiq {
            self.r[8] = self.banked_r8_fiq;
            self.r[9] = self.banked_r9_fiq;
            self.r[10] = self.banked_r10_fiq;
            self.r[11] = self.banked_r11_fiq;
            self.r[12] = self.banked_r12_fiq;
        }
    }

    fn mode_index(&self, mode: Mode) -> usize {
        match mode {
            Mode::Fiq => 0,
            Mode::Irq => 1,
            Mode::Supervisor => 2,
            Mode::Abort => 3,
            Mode::Undefined => 4,
            Mode::System => 5,
            Mode::User => 5,
        }
    }

    fn get_user_reg(&self, n: usize) -> u32 {
        match n {
            8 if self.get_mode() == Mode::Fiq => self.banked_r8_fiq,
            9 if self.get_mode() == Mode::Fiq => self.banked_r9_fiq,
            10 if self.get_mode() == Mode::Fiq => self.banked_r10_fiq,
            11 if self.get_mode() == Mode::Fiq => self.banked_r11_fiq,
            12 if self.get_mode() == Mode::Fiq => self.banked_r12_fiq,
            13 => self.banked_sp[5],
            14 => self.banked_lr[5],
            _ => self.r[n],
        }
    }

    fn set_user_reg(&mut self, n: usize, val: u32) {
        match n {
            8 if self.get_mode() == Mode::Fiq => self.banked_r8_fiq = val,
            9 if self.get_mode() == Mode::Fiq => self.banked_r9_fiq = val,
            10 if self.get_mode() == Mode::Fiq => self.banked_r10_fiq = val,
            11 if self.get_mode() == Mode::Fiq => self.banked_r11_fiq = val,
            12 if self.get_mode() == Mode::Fiq => self.banked_r12_fiq = val,
            13 => self.banked_sp[5] = val,
            14 => self.banked_lr[5] = val,
            _ => self.r[n] = val,
        }
    }

    fn get_spsr(&self) -> u32 {
        let mode = self.get_mode();
        let idx = self.mode_index(mode);
        if idx < 6 {
            self.banked_spsr[idx]
        } else {
            self.cpsr
        }
    }

    fn set_spsr(&mut self, val: u32) {
        let mode = self.get_mode();
        let idx = self.mode_index(mode);
        if idx < 6 {
            self.banked_spsr[idx] = val;
        }
    }

    // Flag access
    pub fn get_flag_n(&self) -> bool {
        self.cpsr & (CpsrFlags::N.bits()) != 0
    }

    pub fn set_flag_n(&mut self, val: bool) {
        if val {
            self.cpsr |= CpsrFlags::N.bits();
        } else {
            self.cpsr &= !CpsrFlags::N.bits();
        }
    }

    pub fn get_flag_z(&self) -> bool {
        self.cpsr & (CpsrFlags::Z.bits()) != 0
    }

    pub fn set_flag_z(&mut self, val: bool) {
        if val {
            self.cpsr |= CpsrFlags::Z.bits();
        } else {
            self.cpsr &= !CpsrFlags::Z.bits();
        }
    }

    pub fn get_flag_c(&self) -> bool {
        self.cpsr & (CpsrFlags::C.bits()) != 0
    }

    pub fn set_flag_c(&mut self, val: bool) {
        if val {
            self.cpsr |= CpsrFlags::C.bits();
        } else {
            self.cpsr &= !CpsrFlags::C.bits();
        }
    }

    pub fn get_flag_v(&self) -> bool {
        self.cpsr & (CpsrFlags::V.bits()) != 0
    }

    pub fn set_flag_v(&mut self, val: bool) {
        if val {
            self.cpsr |= CpsrFlags::V.bits();
        } else {
            self.cpsr &= !CpsrFlags::V.bits();
        }
    }

    // State access
    pub fn is_thumb_mode(&self) -> bool {
        self.cpsr & (CpsrFlags::THUMB.bits()) != 0
    }

    pub fn set_thumb_mode(&mut self, thumb: bool) {
        if thumb {
            self.cpsr |= CpsrFlags::THUMB.bits();
        } else {
            self.cpsr &= !CpsrFlags::THUMB.bits();
        }
    }

    pub fn are_interrupts_enabled(&self) -> bool {
        self.cpsr & (CpsrFlags::IRQ.bits()) == 0
    }

    pub fn set_interrupts_enabled(&mut self, enabled: bool) {
        if enabled {
            self.cpsr &= !CpsrFlags::IRQ.bits();
        } else {
            self.cpsr |= CpsrFlags::IRQ.bits();
        }
    }

    /// Execute one instruction, return cycles taken
    pub fn step(&mut self, mem: &mut super::Memory) -> u32 {
        if self.is_thumb_mode() {
            self.step_thumb(mem)
        } else {
            self.step_arm(mem)
        }
    }

    fn step_arm(&mut self, mem: &mut super::Memory) -> u32 {
        if !self.pipeline_loaded {
            self.pipeline_pc[0] = self.r[15];
            self.pipeline[0] = mem.read_word(self.r[15]);

            let n1 = self.r[15].wrapping_add(4);
            self.pipeline_pc[1] = n1;
            self.pipeline[1] = mem.read_word(n1);

            let n2 = self.r[15].wrapping_add(8);
            self.pipeline_pc[2] = n2;
            self.pipeline[2] = mem.read_word(n2);

            self.pipeline_loaded = true;
        }

        let instruction_pc = self.pipeline_pc[0];
        let opcode = self.pipeline[0];

        self.trace_record(instruction_pc, opcode);

        self.pc_written = false;
        self.r[15] = instruction_pc.wrapping_add(8);

        let cycles = self.execute_arm_with_pc(opcode, mem, instruction_pc, self.r[15]);

        if self.pc_written {
            self.pipeline_loaded = false;
        } else {
            self.pipeline[0] = self.pipeline[1];
            self.pipeline_pc[0] = self.pipeline_pc[1];

            self.pipeline[1] = self.pipeline[2];
            self.pipeline_pc[1] = self.pipeline_pc[2];

            let fetch_pc = instruction_pc.wrapping_add(12);
            self.pipeline_pc[2] = fetch_pc;
            self.pipeline[2] = mem.read_word(fetch_pc);

            self.r[15] = instruction_pc.wrapping_add(4);
        }

        cycles
    }

    fn execute_arm_with_pc(
        &mut self,
        opcode: u32,
        mem: &mut super::Memory,
        instruction_pc: u32,
        _pc_at_execution: u32,
    ) -> u32 {
        // Extract condition field (bits 31-28)
        let cond = ((opcode >> 28) & 0xF) as usize;

        // Check if condition is satisfied
        if !self.check_condition(cond) {
            // Condition not met, skip this instruction
            // Still advance PC and take cycles
            self.r[15] = self.r[15].wrapping_add(4);
            return 1;
        }

        // ARM instruction decoding
        // Bits 27-26: Instruction category
        let category = (opcode >> 26) & 0x3;

        match category {
            0x0 => {
                // Data processing / PSR transfer / Load-store halfword and signed byte
                let _bits_27_25 = (opcode >> 25) & 0x7;

                if (opcode & 0x0FFF_FFF0) == 0x012F_FF10 {
                    // Branch and exchange
                    self.execute_arm_bx(opcode, mem)
                } else if (opcode >> 25) & 0x7 == 0b000 && ((opcode >> 4) & 0xF) != 0x9 {
                    let bit4 = (opcode & 0x0000_0010) != 0;
                    let bit5 = (opcode & 0x0000_0020) != 0;
                    let bit6 = (opcode & 0x0000_0040) != 0;
                    let bit7 = (opcode & 0x0000_0080) != 0;
                    let bit22 = (opcode & 0x0040_0000) != 0;
                    if bit7 && (bit5 || bit6) && (bit4 || bit22) {
                        // Load-store halfword or signed byte
                        // ARM7TDMI: bit7=1 required, bits 6-5=SH, bit4=1(reg) or bit22=1(imm)
                        self.execute_arm_load_store_halfword(opcode, mem)
                    } else if (opcode & 0x0FB0_0FF0) == 0x0100_0090 {
                        // SWP/SWPB - atomic swap
                        self.execute_arm_swp(opcode, mem)
                    } else if (opcode & 0x0190_0000) == 0x0100_0000 {
                        // PSR transfer (bit 24=1, bit 23=0, bit 20=0)
                        self.execute_arm_psr(opcode, mem)
                    } else if (opcode & 0x0F00_00F0) == 0x0000_0090 {
                        // Multiply (MUL, MLA, UMULL, UMLAL, SMULL, SMLAL)
                        self.execute_arm_multiply(opcode)
                    } else {
                        // Data processing
                        self.execute_arm_data_processing(opcode, mem)
                    }
                } else if (opcode & 0x0F00_00F0) == 0x0000_0090 {
                    self.execute_arm_multiply(opcode)
                } else if (opcode & 0x0FB0_0FF0) == 0x0100_0090 {
                    self.execute_arm_swp(opcode, mem)
                } else if (opcode & 0x0190_0000) == 0x0100_0000 {
                    // PSR transfer (bit 24=1, bit 23=0, bit 20=0)
                    self.execute_arm_psr(opcode, mem)
                } else {
                    // Data processing
                    self.execute_arm_data_processing(opcode, mem)
                }
            }
            0x1 => {
                if (opcode >> 25) & 1 != 0 {
                    self.execute_arm_load_store_register(opcode, mem)
                } else {
                    self.execute_arm_load_store(opcode, mem)
                }
            }
            0x2 => {
                // Load/store register offset / LDM / STM / Branch
                // Check bits 27-25 to distinguish sub-types
                let bits_27_25 = (opcode >> 25) & 0x7;

                if bits_27_25 == 0b101 {
                    // Branch (B) or Branch with Link (BL)
                    self.execute_arm_branch(opcode, instruction_pc, mem)
                } else if bits_27_25 == 0b100 {
                    // LDM (Load Multiple) or STM (Store Multiple)
                    self.execute_arm_block_data_transfer(opcode, mem, instruction_pc)
                } else {
                    // Load/store with register offset
                    self.execute_arm_load_store_register(opcode, mem)
                }
            }
            0x3 => {
                // Check for SWI first (bits 27-24 = 0b1111)
                if (opcode & 0x0F00_0000) == 0x0F00_0000 {
                    // SWI: set LR to return address, then handle
                    self.r[14] = instruction_pc + 4;
                    self.execute_arm_swi(opcode, mem)
                } else {
                    // Branch / Branch with link
                    self.execute_arm_branch(opcode, instruction_pc, mem)
                }
            }
            _ => 1, // Unknown, treat as NOP
        }
    }

    fn execute_arm_data_processing(&mut self, opcode: u32, mem: &mut super::Memory) -> u32 {
        let op = (opcode >> 21) & 0xF;
        let s = ((opcode >> 20) & 1) != 0;
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rd = ((opcode >> 12) & 0xF) as usize;
        let operand2 = opcode & 0xFFF;
        let i_bit = ((opcode >> 25) & 1) != 0;
        let register_shift = !i_bit && (operand2 & 0x10) != 0;

        let mut rn_val = self.r[rn];
        if register_shift && rn == 15 {
            rn_val = rn_val.wrapping_add(4);
        }
        let (op2_val, shifter_carry) = self.decode_operand2_ex(operand2, i_bit, register_shift);

        #[cfg(debug_assertions)]
        let r12_before = self.r[12];

        match op {
            0x0 => {
                // AND
                let result = rn_val & op2_val;
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(shifter_carry);
                }
            }
            0x1 => {
                // EOR
                let result = rn_val ^ op2_val;
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(shifter_carry);
                }
            }
            0x2 => {
                // SUB
                let (result, overflow) = rn_val.overflowing_sub(op2_val);
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(!overflow);
                    self.set_flag_v(((rn_val as i32) < (op2_val as i32)) ^ ((result as i32) < 0));
                }
            }
            0x3 => {
                // RSB
                let (result, overflow) = op2_val.overflowing_sub(rn_val);
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(!overflow);
                    self.set_flag_v(((op2_val as i32) < (rn_val as i32)) ^ ((result as i32) < 0));
                }
            }
            0x4 => {
                // ADD
                let (result, overflow) = rn_val.overflowing_add(op2_val);
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(overflow);
                    self.set_flag_v(
                        ((rn_val as i32) > 0 && (op2_val as i32) > 0 && (result as i32) < 0)
                            || ((rn_val as i32) < 0 && (op2_val as i32) < 0 && (result as i32) > 0),
                    );
                }
            }
            0x5 => {
                // ADC
                let c = if self.get_flag_c() { 1 } else { 0 };
                let (result1, overflow1) = rn_val.overflowing_add(op2_val);
                let (result, overflow2) = result1.overflowing_add(c);
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(overflow1 || overflow2);
                    // Signed overflow: if A and B have same sign but result differs
                    let v = (!((rn_val ^ op2_val) >> 31 != 0)) && ((rn_val ^ result) >> 31 != 0);
                    self.set_flag_v(v);
                }
            }
            0x6 => {
                // SBC (Subtract with Carry)
                // SBC subtracts (op2 + NOT carry) from rn
                let c = if self.get_flag_c() { 0 } else { 1 }; // NOT carry (borrow)
                let borrow = op2_val.wrapping_add(c);
                let (result, overflow) = rn_val.overflowing_sub(borrow);
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(!overflow);
                    self.set_flag_v(((rn_val as i32) < (borrow as i32)) ^ ((result as i32) < 0));
                }
            }
            0x7 => {
                // RSC (Reverse Subtract with Carry)
                let c = if self.get_flag_c() { 0 } else { 1 }; // NOT carry (borrow)
                let borrow = rn_val.wrapping_add(c);
                let (result, overflow) = op2_val.overflowing_sub(borrow);
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(!overflow);
                    self.set_flag_v(((op2_val as i32) < (borrow as i32)) ^ ((result as i32) < 0));
                }
            }
            0x8 => {
                // TST - always sets flags
                let result = rn_val & op2_val;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(shifter_carry);
            }
            0x9 => {
                // TEQ - always sets flags
                let result = rn_val ^ op2_val;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(shifter_carry);
            }
            0xA => {
                // CMP - always sets flags
                let (result, overflow) = rn_val.overflowing_sub(op2_val);
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(!overflow);
                self.set_flag_v(((rn_val as i32) < (op2_val as i32)) ^ ((result as i32) < 0));
            }
            0xB => {
                // CMN - always sets flags
                let (result, overflow) = rn_val.overflowing_add(op2_val);
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(overflow);
                self.set_flag_v(
                    ((rn_val as i32) > 0 && (op2_val as i32) > 0 && (result as i32) < 0)
                        || ((rn_val as i32) < 0 && (op2_val as i32) < 0 && (result as i32) > 0),
                );
            }
            0xC => {
                // ORR
                let result = rn_val | op2_val;
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(shifter_carry);
                }
            }
            0xD => {
                // MOV
                if rd == 15 {
                    if s {
                        let spsr = self.get_spsr();
                        self.cpsr = spsr;
                        self.set_mode(self.get_mode());
                    } else if self.get_mode() == Mode::Irq {
                        let old_mode = self.get_mode();
                        let spsr = self.banked_spsr[2];
                        let new_mode = Mode::from_bits(spsr);
                        self.cpsr = spsr;
                        if old_mode != new_mode {
                            let old_idx = self.mode_index(old_mode);
                            if old_idx < 6 {
                                self.banked_sp[old_idx] = self.r[13];
                                self.banked_lr[old_idx] = self.r[14];
                            }
                            let new_idx = self.mode_index(new_mode);
                            if new_idx < 6 {
                                self.r[13] = self.banked_sp[new_idx];
                                self.r[14] = self.banked_lr[new_idx];
                            }
                            if new_mode == Mode::Fiq {
                                self.banked_r8_fiq = self.r[8];
                                self.banked_r9_fiq = self.r[9];
                                self.banked_r10_fiq = self.r[10];
                                self.banked_r11_fiq = self.r[11];
                                self.banked_r12_fiq = self.r[12];
                            } else if old_mode == Mode::Fiq {
                                self.r[8] = self.banked_r8_fiq;
                                self.r[9] = self.banked_r9_fiq;
                                self.r[10] = self.banked_r10_fiq;
                                self.r[11] = self.banked_r11_fiq;
                                self.r[12] = self.banked_r12_fiq;
                            }
                        }
                        mem.set_bios_read_return(0xE55EC002);
                    }
                    let thumb = (op2_val & 1) != 0;
                    self.set_pc(op2_val & 0xFFFFFFFE);
                    self.set_thumb_mode(thumb);
                    return 1;
                } else {
                    self.r[rd] = op2_val;
                    if s {
                        self.set_flag_n((op2_val as i32) < 0);
                        self.set_flag_z(op2_val == 0);
                        self.set_flag_c(shifter_carry);
                    }
                }
            }
            0xE => {
                // BIC
                let result = rn_val & !op2_val;
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(shifter_carry);
                }
            }
            0xF => {
                // MVN
                let result = !op2_val;
                self.r[rd] = result;
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(shifter_carry);
                }
            }
            _ => {}
        }

        // CMP/CMN/TST/TEQ with Rd=15 and S=1: restore CPSR from SPSR (no PC write)
        if rd == 15 && s && (op == 0x8 || op == 0x9 || op == 0xA || op == 0xB) {
            let old_mode = self.get_mode();
            let spsr = self.get_spsr();
            let new_mode = Mode::from_bits(spsr);
            self.cpsr = spsr;
            if old_mode != new_mode {
                let old_idx = self.mode_index(old_mode);
                if old_idx < 6 {
                    self.banked_sp[old_idx] = self.r[13];
                    self.banked_lr[old_idx] = self.r[14];
                    self.banked_spsr[old_idx] = spsr;
                }
                let new_idx = self.mode_index(new_mode);
                if new_idx < 6 {
                    self.r[13] = self.banked_sp[new_idx];
                    self.r[14] = self.banked_lr[new_idx];
                }
                if new_mode == Mode::Fiq {
                    self.banked_r8_fiq = self.r[8];
                    self.banked_r9_fiq = self.r[9];
                    self.banked_r10_fiq = self.r[10];
                    self.banked_r11_fiq = self.r[11];
                    self.banked_r12_fiq = self.r[12];
                } else if old_mode == Mode::Fiq {
                    self.r[8] = self.banked_r8_fiq;
                    self.r[9] = self.banked_r9_fiq;
                    self.r[10] = self.banked_r10_fiq;
                    self.r[11] = self.banked_r11_fiq;
                    self.r[12] = self.banked_r12_fiq;
                }
            }
        }

        // Check if PC was the destination register (but not for test/comparison ops)
        // TST (0x8), TEQ (0x9), CMP (0xA), CMN (0xB) don't write to rd
        if rd == 15 && op != 0x8 && op != 0x9 && op != 0xA && op != 0xB {
            let result = self.r[15];
            if s {
                let old_mode = self.get_mode();
                let spsr = self.get_spsr();
                let new_mode = Mode::from_bits(spsr);
                self.cpsr = spsr;
                if old_mode != new_mode {
                    let old_idx = self.mode_index(old_mode);
                    if old_idx < 6 {
                        self.banked_sp[old_idx] = self.r[13];
                        self.banked_lr[old_idx] = self.r[14];
                        self.banked_spsr[old_idx] = spsr;
                    }
                    let new_idx = self.mode_index(new_mode);
                    if new_idx < 6 {
                        self.r[13] = self.banked_sp[new_idx];
                        self.r[14] = self.banked_lr[new_idx];
                    }
                    if new_mode == Mode::Fiq {
                        self.banked_r8_fiq = self.r[8];
                        self.banked_r9_fiq = self.r[9];
                        self.banked_r10_fiq = self.r[10];
                        self.banked_r11_fiq = self.r[11];
                        self.banked_r12_fiq = self.r[12];
                    } else if old_mode == Mode::Fiq {
                        self.r[8] = self.banked_r8_fiq;
                        self.r[9] = self.banked_r9_fiq;
                        self.r[10] = self.banked_r10_fiq;
                        self.r[11] = self.banked_r11_fiq;
                        self.r[12] = self.banked_r12_fiq;
                    }
                }
            }
            let thumb = (result & 1) != 0;
            self.set_pc(result & 0xFFFFFFFE);
            self.set_thumb_mode(thumb);
            return 1;
        }

        self.r[15] = self.r[15].wrapping_add(4);

        // Debug: check if R12 changed
        #[cfg(debug_assertions)]
        if self.r[12] != r12_before {
            eprintln!("R12 CHANGED in data processing op=0x{:X}, rd={}, opcode=0x{:08X}: 0x{:08X} -> 0x{:08X}",
                     op, rd, opcode, r12_before, self.r[12]);
        }

        1
    }

    #[allow(dead_code)]
    fn decode_operand2(&self, operand2: u32, is_immediate: bool) -> (u32, bool) {
        self.decode_operand2_ex(operand2, is_immediate, false)
    }

    fn decode_operand2_ex(
        &self,
        operand2: u32,
        is_immediate: bool,
        is_reg_shift_instr: bool,
    ) -> (u32, bool) {
        let shift = (operand2 >> 4) & 0xFF;

        if is_immediate {
            let imm8 = (operand2 & 0xFF) as u32;
            let rotate = ((operand2 >> 8) & 0xF) * 2;
            let result = if rotate == 0 {
                imm8
            } else {
                imm8.rotate_right(rotate)
            };
            let carry = if rotate == 0 {
                self.get_flag_c()
            } else {
                (result >> 31) != 0
            };
            (result, carry)
        } else {
            let rm = (operand2 & 0xF) as usize;
            let is_register_shift = (operand2 & 0x10) != 0;
            let mut val = self.r[rm];
            if is_register_shift && is_reg_shift_instr && rm == 15 {
                val = val.wrapping_add(4);
            }

            if is_register_shift {
                let rs = ((operand2 >> 8) & 0xF) as usize;
                let amount = (self.r[rs] & 0xFF) as u32;
                let shift_type = (shift >> 1) & 0x3;

                let (result, carry) = match shift_type {
                    0 => {
                        if amount == 0 {
                            (val, self.get_flag_c())
                        } else if amount < 32 {
                            (val << amount, (val >> (32 - amount)) != 0)
                        } else {
                            (0, amount == 32 && (val & 1) != 0)
                        }
                    }
                    1 => {
                        if amount == 0 {
                            (val, self.get_flag_c())
                        } else if amount < 32 {
                            (val >> amount, (val >> (amount - 1)) & 1 != 0)
                        } else {
                            (0, amount == 32 && (val >> 31) != 0)
                        }
                    }
                    2 => {
                        if amount == 0 {
                            (val, self.get_flag_c())
                        } else if amount < 32 {
                            (
                                ((val as i32) >> amount) as u32,
                                (val >> (amount - 1)) & 1 != 0,
                            )
                        } else {
                            let bit31 = (val >> 31) != 0;
                            (if bit31 { 0xFFFFFFFF } else { 0 }, bit31)
                        }
                    }
                    3 => {
                        if amount == 0 {
                            (val, self.get_flag_c())
                        } else {
                            let r = (amount & 0x1F) as u32;
                            if r == 0 {
                                (val, (val >> 31) != 0)
                            } else {
                                let result = val.rotate_right(r);
                                (result, (result >> 31) != 0)
                            }
                        }
                    }
                    _ => (val, self.get_flag_c()),
                };
                (result, carry)
            } else {
                let shift_type = (shift >> 1) & 0x3;
                let shift_imm = (shift >> 3) & 0x1F;

                let (result, carry) = match shift_type {
                    0 => {
                        if shift_imm == 0 {
                            (val, self.get_flag_c())
                        } else {
                            ((val << shift_imm), (val >> (32 - shift_imm)) != 0)
                        }
                    }
                    1 => {
                        if shift_imm == 0 {
                            (0, (val >> 31) != 0)
                        } else {
                            (val >> shift_imm, (val >> (shift_imm - 1)) & 1 != 0)
                        }
                    }
                    2 => {
                        if shift_imm == 0 {
                            let bit31 = (val >> 31) != 0;
                            (if bit31 { 0xFFFFFFFF } else { 0 }, bit31)
                        } else {
                            (
                                ((val as i32) >> shift_imm) as u32,
                                (val >> (shift_imm - 1)) & 1 != 0,
                            )
                        }
                    }
                    3 => {
                        if shift_imm == 0 {
                            let c = self.get_flag_c();
                            ((val >> 1) | if c { 0x80000000 } else { 0 }, val & 1 != 0)
                        } else {
                            let result = val.rotate_right(shift_imm);
                            (result, (val >> (shift_imm - 1)) & 1 != 0)
                        }
                    }
                    _ => (val, self.get_flag_c()),
                };
                (result, carry)
            }
        }
    }

    #[allow(dead_code)]
    fn set_flags_from_result(&mut self, result: u32) {
        self.set_flag_n((result as i32) < 0);
        self.set_flag_z(result == 0);
        // C and V depend on the operation
    }

    fn execute_arm_multiply(&mut self, opcode: u32) -> u32 {
        let rd = ((opcode >> 16) & 0xF) as usize;
        let rn = ((opcode >> 12) & 0xF) as usize;
        let rs = ((opcode >> 8) & 0xF) as usize;
        let rm = (opcode & 0xF) as usize;
        let s = ((opcode >> 20) & 1) != 0;
        let a = ((opcode >> 21) & 1) != 0; // Accumulate
        let long = ((opcode >> 23) & 1) != 0; // Long multiply
        let u = ((opcode >> 22) & 1) != 0; // Signed (for long)

        if long {
            // 64-bit result: RdHi:RdLo = Rm * Rs [+ Rn:Rd]
            let rd_hi = rd;
            let rd_lo = rn;

            let result = if u {
                // Signed long multiply
                let rm_signed = self.r[rm] as i32 as i64;
                let rs_signed = self.r[rs] as i32 as i64;
                let product = rm_signed.wrapping_mul(rs_signed) as u64;
                if a {
                    let acc = ((self.r[rd_hi] as u64) << 32) | (self.r[rd_lo] as u64);
                    product.wrapping_add(acc)
                } else {
                    product
                }
            } else {
                // Unsigned long multiply
                let product = (self.r[rm] as u64).wrapping_mul(self.r[rs] as u64);
                if a {
                    let acc = ((self.r[rd_hi] as u64) << 32) | (self.r[rd_lo] as u64);
                    product.wrapping_add(acc)
                } else {
                    product
                }
            };

            self.r[rd_hi] = (result >> 32) as u32;
            self.r[rd_lo] = (result & 0xFFFF_FFFF) as u32;

            if s {
                self.set_flag_n((result as i64) < 0);
                self.set_flag_z(result == 0);
            }
        } else {
            // 32-bit result: Rd = Rm * Rs [+ Rn]
            let product = (self.r[rm] as u32).wrapping_mul(self.r[rs] as u32);
            let result = if a {
                product.wrapping_add(self.r[rn])
            } else {
                product
            };
            self.r[rd] = result;

            if s {
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
        }

        self.r[15] = self.r[15].wrapping_add(4);
        1
    }

    fn execute_arm_swp(&mut self, opcode: u32, mem: &mut super::Memory) -> u32 {
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rd = ((opcode >> 12) & 0xF) as usize;
        let rm = (opcode & 0xF) as usize;
        let byte = ((opcode >> 22) & 1) != 0;

        let addr = self.r[rn];

        if byte {
            let old_val = mem.read_byte(addr) as u32;
            mem.write_byte(addr, self.r[rm] as u8);
            self.r[rd] = old_val;
        } else {
            let old_val = mem.read_word(addr);
            mem.write_word(addr, self.r[rm]);
            self.r[rd] = old_val;
        }

        self.r[15] = self.r[15].wrapping_add(4);
        3
    }

    fn execute_arm_psr(&mut self, opcode: u32, _mem: &mut super::Memory) -> u32 {
        // Note: We don't have instruction_pc here, so we can't log it accurately
        // The logging will be done at the call site

        let is_mrs = (opcode & (1 << 21)) == 0;
        let psr = (opcode & (1 << 22)) != 0;

        if is_mrs {
            let rd = ((opcode >> 12) & 0xF) as usize;
            if psr {
                self.r[rd] = self.get_spsr();
            } else {
                self.r[rd] = self.cpsr;
            }
        } else {
            let rm = (opcode & 0xF) as usize;
            let immediate = (opcode & (1 << 25)) != 0;

            let val = if immediate {
                let imm = opcode & 0xFF;
                let rotate = ((opcode >> 8) & 0xF) * 2;
                imm.rotate_right(rotate) as u32
            } else {
                self.r[rm]
            };

            let apply_flags = (opcode & 0x80000) != 0;
            let apply_status = (opcode & 0x40000) != 0;
            let apply_extension = (opcode & 0x20000) != 0;
            let apply_control = (opcode & 0x10000) != 0;

            if psr {
                let mut spsr = self.get_spsr();
                if apply_flags {
                    spsr = (spsr & !0xF0000000) | (val & 0xF0000000);
                }
                if self.get_mode() != Mode::User {
                    if apply_status {
                        spsr = (spsr & !0x00FF0000) | (val & 0x00FF0000);
                    }
                    if apply_extension {
                        spsr = (spsr & !0x0000FF00) | (val & 0x0000FF00);
                    }
                    if apply_control {
                        spsr = (spsr & !0x000000FF) | (val & 0x000000FF);
                    }
                }
                self.set_spsr(spsr);
            } else {
                let old_mode = self.get_mode();
                if apply_flags {
                    self.cpsr = (self.cpsr & !0xF0000000) | (val & 0xF0000000);
                }
                if old_mode != Mode::User {
                    if apply_status {
                        self.cpsr = (self.cpsr & !0x00FF0000) | (val & 0x00FF0000);
                    }
                    if apply_extension {
                        self.cpsr = (self.cpsr & !0x0000FF00) | (val & 0x0000FF00);
                    }
                    if apply_control {
                        self.cpsr = (self.cpsr & !0x000000FF) | (val & 0x000000FF);
                        let new_mode = Mode::from_bits(self.cpsr);
                        if new_mode != old_mode {
                            let _saved_cpsr = self.cpsr;
                            self.cpsr = (self.cpsr & !0x1F) | (old_mode as u32);
                            self.set_mode(new_mode);
                        }
                    }
                }
            }
        }

        self.r[15] = self.r[15].wrapping_add(4);
        1
    }

    fn execute_arm_load_store_halfword(&mut self, opcode: u32, mem: &mut super::Memory) -> u32 {
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rd = ((opcode >> 12) & 0xF) as usize;
        let load = ((opcode >> 20) & 1) != 0;
        let writeback = ((opcode >> 21) & 1) != 0;
        let is_immediate = ((opcode >> 22) & 1) != 0;
        let up = ((opcode >> 23) & 1) != 0;
        let pre_index = ((opcode >> 24) & 1) != 0;

        let offset = if is_immediate {
            let imm4h = ((opcode >> 8) & 0xF) as u32;
            let imm4l = (opcode & 0xF) as u32;
            (imm4h << 4) | imm4l
        } else {
            let rm = (opcode & 0xF) as usize;
            self.r[rm]
        };

        let base = self.r[rn];
        let offset_addr = if up {
            base.wrapping_add(offset)
        } else {
            base.wrapping_sub(offset)
        };
        let addr = if pre_index { offset_addr } else { base };

        // Perform load or store
        if load {
            let is_signed = ((opcode >> 6) & 1) != 0; // S bit
            let is_halfword = ((opcode >> 5) & 1) != 0; // H bit

            let val = if is_signed {
                if is_halfword {
                    if addr & 1 != 0 {
                        mem.read_half_rotated(addr) as u8 as i8 as i32 as u32
                    } else {
                        mem.read_half(addr) as i16 as i32 as u32
                    }
                } else {
                    mem.read_byte(addr) as i8 as i32 as u32
                }
            } else {
                mem.read_half_rotated(addr)
            };

            if !pre_index || writeback {
                self.r[rn] = offset_addr;
            }

            self.r[rd] = val;
        } else {
            mem.write_half(addr, self.r[rd] as u16);

            if !pre_index || writeback {
                self.r[rn] = offset_addr;
            }
        }

        self.r[15] = self.r[15].wrapping_add(4);
        1
    }

    fn execute_arm_bx(&mut self, opcode: u32, mem: &mut super::Memory) -> u32 {
        let rm = (opcode & 0xF) as usize;
        let target = self.r[rm];

        #[cfg(debug_assertions)]
        eprintln!(
            "BX: R{} = 0x{:08X}, Thumb bit = {}",
            rm,
            target,
            (target & 1) != 0
        );

        if self.get_mode() == Mode::Irq {
            let old_mode = self.get_mode();
            let spsr = self.banked_spsr[2];
            let new_mode = Mode::from_bits(spsr);
            self.cpsr = spsr;
            if old_mode != new_mode {
                let old_idx = self.mode_index(old_mode);
                if old_idx < 6 {
                    self.banked_sp[old_idx] = self.r[13];
                    self.banked_lr[old_idx] = self.r[14];
                }
                let new_idx = self.mode_index(new_mode);
                if new_idx < 6 {
                    self.r[13] = self.banked_sp[new_idx];
                    self.r[14] = self.banked_lr[new_idx];
                }
                if new_mode == Mode::Fiq {
                    self.banked_r8_fiq = self.r[8];
                    self.banked_r9_fiq = self.r[9];
                    self.banked_r10_fiq = self.r[10];
                    self.banked_r11_fiq = self.r[11];
                    self.banked_r12_fiq = self.r[12];
                } else if old_mode == Mode::Fiq {
                    self.r[8] = self.banked_r8_fiq;
                    self.r[9] = self.banked_r9_fiq;
                    self.r[10] = self.banked_r10_fiq;
                    self.r[11] = self.banked_r11_fiq;
                    self.r[12] = self.banked_r12_fiq;
                }
            }
            mem.set_bios_read_return(0xE55EC002);
        }

        self.set_thumb_mode((target & 1) != 0);
        self.set_pc(target);

        2
    }

    fn execute_arm_load_store(&mut self, _opcode: u32, mem: &mut super::Memory) -> u32 {
        let rn = ((_opcode >> 16) & 0xF) as usize;
        let rd = ((_opcode >> 12) & 0xF) as usize;
        let offset = (_opcode & 0xFFF) as i32 as i64;

        let base = self.r[rn] as i64;

        let load = (_opcode >> 20) & 1 != 0;
        let byte = (_opcode >> 22) & 1 != 0;
        let writeback = (_opcode >> 21) & 1 != 0;
        let add = (_opcode >> 23) & 1 != 0;
        let pre_index = (_opcode >> 24) & 1 != 0;
        let u = if add { 1i64 } else { -1i64 };

        let offset_addr = (base + u * offset) as u32;
        let addr = if pre_index { offset_addr } else { base as u32 };

        if load {
            let val = if byte {
                mem.read_byte(addr) as u32
            } else {
                mem.read_word(addr)
            };

            if !pre_index || writeback {
                self.r[rn] = offset_addr;
            }

            if rd == 15 {
                self.set_pc(val & 0xFFFFFFFE);
                return 2;
            } else {
                self.r[rd] = val;
            }
        } else {
            let val = if rd == 15 {
                self.r[rd].wrapping_add(4)
            } else {
                self.r[rd]
            };
            if byte {
                mem.write_byte(addr, val as u8);
            } else {
                mem.write_word(addr, val);
            }

            if !pre_index || writeback {
                self.r[rn] = offset_addr;
            }
        }

        self.r[15] = self.r[15].wrapping_add(4);
        2
    }

    fn execute_arm_load_store_register(&mut self, opcode: u32, mem: &mut super::Memory) -> u32 {
        let load = ((opcode >> 20) & 1) != 0;
        let byte = ((opcode >> 22) & 1) != 0;
        let writeback = ((opcode >> 21) & 1) != 0;
        let _pre_index = ((opcode >> 24) & 1) != 0;
        let add = ((opcode >> 23) & 1) != 0;
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rd = ((opcode >> 12) & 0xF) as usize;
        let rm = (opcode & 0xF) as usize;

        let shift_type = (opcode >> 5) & 0x3;
        let shift_amount = ((opcode >> 7) & 0x1F) as u32;

        let mut offset = self.r[rm];
        match shift_type {
            0 => offset <<= shift_amount,
            1 => {
                if shift_amount == 0 {
                    offset = 0;
                } else {
                    offset >>= shift_amount;
                }
            }
            2 => offset = ((offset as i32) >> shift_amount) as u32,
            3 => {
                if shift_amount == 0 {
                    let c = if self.get_flag_c() { 1u32 << 31 } else { 0 };
                    offset = c | (offset >> 1);
                } else {
                    offset = offset.rotate_right(shift_amount);
                }
            }
            _ => {}
        }

        let base = self.r[rn] as i64;
        let addr = if add {
            (base + offset as i64) as u32
        } else {
            (base - offset as i64) as u32
        };

        if load {
            let val = if byte {
                mem.read_byte(addr) as u32
            } else {
                mem.read_word(addr)
            };

            if writeback {
                self.r[rn] = addr;
            }

            if rd == 15 {
                self.set_pc(val & 0xFFFFFFFE);
                return 2;
            } else {
                self.r[rd] = val;
            }
        } else {
            let val = if rd == 15 {
                self.r[rd].wrapping_add(4)
            } else {
                self.r[rd]
            };
            if byte {
                mem.write_byte(addr, val as u8);
            } else {
                mem.write_word(addr, val);
            }

            if writeback {
                self.r[rn] = addr;
            }
        }

        self.r[15] = self.r[15].wrapping_add(4);
        2
    }

    fn execute_arm_block_data_transfer(
        &mut self,
        opcode: u32,
        mem: &mut super::Memory,
        _instruction_pc: u32,
    ) -> u32 {
        // LDM (Load Multiple) or STM (Store Multiple)
        // Format: PUWL (bits 24-21) + Rn (bits 19-16) + register list (bits 15-0)

        let pre_index = ((opcode >> 24) & 1) != 0; // P bit
        let add_to_base = ((opcode >> 23) & 1) != 0; // U bit (1=add, 0=subtract)
        let force_user = ((opcode >> 22) & 1) != 0; // S bit (^ = user mode registers)
        let writeback = ((opcode >> 21) & 1) != 0; // W bit
        let load = ((opcode >> 20) & 1) != 0; // L bit (1=load/LDM, 0=store/STM)
        let rn = ((opcode >> 16) & 0xF) as usize; // Base register
        let reg_list = opcode & 0xFFFF; // Bitmask of registers

        let mut addr = self.r[rn] & !3;
        if add_to_base {
            // Increment mode (U=1)
            if pre_index {
                // IB: start at base+4
                addr = addr.wrapping_add(4);
            }
            // IA: start at base (no adjustment)
        } else {
            // Decrement mode (U=0)
            if pre_index {
                // DB: start at base - reg_count*4
                let reg_count = reg_list.count_ones() as u32;
                addr = addr.wrapping_sub(reg_count * 4);
            } else {
                // DA: start at base - (reg_count-1)*4
                let reg_count = reg_list.count_ones() as u32;
                if reg_count > 0 {
                    addr = addr.wrapping_sub((reg_count - 1) * 4);
                }
            }
        }

        // Handle empty register list special case
        if reg_list == 0 {
            // Empty rlist: treat as 16 registers for address calc
            let mut empty_addr = self.r[rn] & !3;
            if add_to_base {
                if pre_index {
                    empty_addr = empty_addr.wrapping_add(4);
                }
            } else {
                if pre_index {
                    empty_addr = empty_addr.wrapping_sub(16 * 4);
                } else {
                    empty_addr = empty_addr.wrapping_sub(15 * 4);
                }
            }
            if load {
                let val = mem.read_word(empty_addr);
                self.set_pc(val);
            } else {
                mem.write_word(empty_addr, self.r[15].wrapping_add(4));
                self.r[15] = self.r[15].wrapping_add(4);
            }
            if writeback {
                if add_to_base {
                    self.r[rn] = self.r[rn].wrapping_add(0x40);
                } else {
                    self.r[rn] = self.r[rn].wrapping_sub(0x40);
                }
            }
            return 3;
        }

        // Process each register
        let is_privileged = self.get_mode() != Mode::User;
        let lowest_reg = reg_list.trailing_zeros() as usize;
        let reg_count = reg_list.count_ones() as u32;
        let wb_value = if add_to_base {
            self.r[rn].wrapping_add(reg_count * 4)
        } else {
            self.r[rn].wrapping_sub(reg_count * 4)
        };

        for reg_idx in 0..16 {
            if reg_list & (1 << reg_idx) != 0 {
                if load {
                    let val = mem.read_word(addr);
                    if force_user && reg_idx != 15 && is_privileged {
                        self.set_user_reg(reg_idx, val);
                    } else {
                        self.r[reg_idx] = val;
                    }
                } else {
                    let val = if reg_idx == 15 {
                        self.r[15].wrapping_add(4)
                    } else if reg_idx == rn && reg_idx != lowest_reg && writeback {
                        wb_value
                    } else if force_user && is_privileged {
                        self.get_user_reg(reg_idx)
                    } else {
                        self.r[reg_idx]
                    };
                    mem.write_word(addr, val);
                }
                addr = addr.wrapping_add(4);
            }
        }

        if writeback && !(load && reg_list & (1 << rn) != 0) {
            let reg_count = reg_list.count_ones() as u32;
            if add_to_base {
                self.r[rn] = self.r[rn].wrapping_add(reg_count * 4);
            } else {
                self.r[rn] = self.r[rn].wrapping_sub(reg_count * 4);
            }
        }

        if load && (reg_list & (1 << 15)) != 0 {
            let pc_value = self.r[15];
            if force_user && is_privileged {
                let spsr = self.get_spsr();
                self.cpsr = spsr;
                self.set_mode(Mode::from_bits(spsr));
            }
            if pc_value & 1 != 0 {
                self.set_thumb_mode(true);
                self.set_pc(pc_value);
            } else {
                self.set_pc(pc_value);
            }
            return 3;
        }

        // Increment PC by 4 (normal behavior)
        self.r[15] = self.r[15].wrapping_add(4);
        3
    }

    fn execute_arm_branch(
        &mut self,
        opcode: u32,
        instruction_pc: u32,
        _mem: &mut super::Memory,
    ) -> u32 {
        // Extract and sign-extend the 24-bit offset
        let offset_imm = (opcode & 0x00FFFFFF) as i32;
        let offset = if (offset_imm & 0x800000) != 0 {
            // Negative: sign extend
            (((offset_imm as u32) | 0xFF000000) as i32) << 2
        } else {
            // Positive
            offset_imm << 2
        };

        let link = ((opcode >> 24) & 1) != 0;

        // DEBUG: Log all branches

        // instruction_pc is the address of the instruction being executed
        // The branch offset is relative to this address
        if link {
            // Return address is the next instruction (instruction_pc + 4)
            self.set_lr(instruction_pc.wrapping_add(4));
        }

        // Calculate branch target
        let target = instruction_pc.wrapping_add(8).wrapping_add(offset as u32);

        #[cfg(debug_assertions)]
        if instruction_pc >= 0x08000100 && instruction_pc <= 0x08000120 {
            eprintln!("BRANCH: target=0x{:08X}, setting PC", target);
        }

        // Set PC using set_pc which aligns to word boundary
        self.set_pc(target);

        2 // Branch takes 2 cycles
    }

    fn execute_arm_swi(&mut self, opcode: u32, mem: &mut super::Memory) -> u32 {
        mem.set_bios_read_return(0xE3A02004);

        let swi_func = opcode & 0xFFFFFF;

        // For Thumb mode, SWI number is in R7 (but we handle Thumb SWI separately)

        // Handle BIOS function calls
        // GBA BIOS uses the upper byte of the SWI comment field (bits 23-16)
        // Some ROMs also use the lower byte (bits 7-0)
        // Try upper byte first, then fall back to lower byte
        let swi_num = if (swi_func >> 16) != 0 {
            (swi_func >> 16) & 0xFF
        } else {
            swi_func & 0xFF
        };
        match swi_num {
            0x00 => {
                // SoftReset - reset the system
                self.reset();
                self.set_pc(0x08000000);
            }
            0x01 => {
                // RegisterRamReset - reset memory regions
                // Simplified: just return
                self.r[15] = self.r[14];
            }
            0x02 | 0x03 => {
                // Halt / Stop - not really implementable without interrupts
                self.r[15] = self.r[14];
            }
            0x04 => {
                // IntrWait - wait for interrupt
                // For now, just return (won't actually wait)
                self.r[15] = self.r[14];
            }
            0x05 => {
                // VBlankIntrWait - wait for VBlank interrupt
                // For now, just return (won't actually wait)
                self.r[15] = self.r[14];
            }
            0x06 => {
                // Div - signed division
                let r0 = self.r[0] as i32;
                let r1 = self.r[1] as i32;
                if r1 != 0 {
                    self.r[0] = r0.wrapping_div(r1) as u32;
                    self.r[3] = r0.wrapping_rem(r1) as u32;
                } else {
                    self.r[0] = if r0 >= 0 { 0x7FFFFFFF } else { 0x80000000 };
                    self.r[3] = self.r[0];
                }
                self.r[15] = self.r[14];
            }
            0x07 | 0x08 => {
                // DivArm / Sqrt
                if (swi_func & 0xFF) == 0x07 {
                    // DivArm: same as Div but R0 and R1 are swapped
                    let r0 = self.r[1] as i32;
                    let r1 = self.r[0] as i32;
                    if r1 != 0 {
                        self.r[1] = r0.wrapping_div(r1) as u32;
                        self.r[3] = r0.wrapping_rem(r1) as u32;
                    } else {
                        self.r[1] = if r0 >= 0 { 0x7FFFFFFF } else { 0x80000000 };
                        self.r[3] = self.r[1];
                    }
                    self.r[15] = self.r[14];
                } else {
                    // Sqrt - unsigned square root
                    let r0 = self.r[0] as f64;
                    self.r[0] = (r0.sqrt()) as u32;
                    self.r[15] = self.r[14];
                }
            }
            0x0E => {
                // Sqrt - square root
                let r0 = self.r[0] as f64;
                self.r[0] = (r0.sqrt()) as u32;
                self.r[15] = self.r[14];
            }
            0x0B => {
                // CpuSet - copy/fill memory
                let src = self.r[0];
                let dst = self.r[1];
                let control = self.r[2];
                let word_size = if (control & 0x04000000) != 0 { 4 } else { 2 };
                let count = (control & 0x000FFFFF) as usize;
                let fill = (control & 0x01000000) != 0;

                for i in 0..count {
                    let s = if fill {
                        src
                    } else {
                        src.wrapping_add((i as u32) * word_size)
                    };
                    let d = dst.wrapping_add((i as u32) * word_size);
                    if word_size == 4 {
                        let val = mem.read_word(s);
                        mem.write_word(d, val);
                    } else {
                        let val = mem.read_half(s) as u32;
                        mem.write_half(d, val as u16);
                    }
                }
                self.r[15] = self.r[14];
            }
            _ => {
                // Unknown SWI - just return
                self.r[15] = self.r[14];
            }
        }

        3 + 3 // SWI takes 3 cycles + 3 for return
    }

    fn step_thumb(&mut self, mem: &mut super::Memory) -> u32 {
        // Load pipeline if needed
        if !self.pipeline_loaded {
            let pc = self.r[15];
            self.pipeline_pc[0] = pc;
            self.pipeline[0] = mem.read_half(pc) as u32;
            self.r[15] = self.r[15].wrapping_add(2);

            self.pipeline_pc[1] = self.r[15];
            self.pipeline[1] = mem.read_half(self.r[15]) as u32;
            self.r[15] = self.r[15].wrapping_add(2);

            self.pipeline_pc[2] = self.r[15];
            self.pipeline[2] = mem.read_half(self.r[15]) as u32;
            self.r[15] = self.r[15].wrapping_add(2);

            self.pipeline_loaded = true;
        }

        // Execute instruction, shift pipeline
        let opcode = self.pipeline[0] as u16;
        let instruction_pc = self.pipeline_pc[0];
        let pc_at_execution = self.r[15];

        self.trace_record(instruction_pc, opcode as u32);

        self.pipeline[0] = self.pipeline[1];
        self.pipeline_pc[0] = self.pipeline_pc[1];

        self.pipeline[1] = self.pipeline[2];
        self.pipeline_pc[1] = self.pipeline_pc[2];

        // Decode and execute
        let cycles = self.execute_thumb(opcode, mem, instruction_pc);

        // Only fetch next instruction if PC wasn't modified
        if self.r[15] == pc_at_execution.wrapping_add(2) {
            let next_pc = self.pipeline_pc[1].wrapping_add(2);
            self.pipeline_pc[2] = next_pc;
            self.pipeline[2] = mem.read_half(next_pc) as u32;
            self.r[15] = next_pc;
        } else {
            self.pipeline_loaded = false;
        }

        cycles
    }

    fn execute_thumb(&mut self, opcode: u16, mem: &mut super::Memory, instruction_pc: u32) -> u32 {
        // Thumb instruction decoding
        // Bits 15-13 determine the instruction category
        let category = (opcode >> 13) & 0x7;

        match category {
            0b000 => {
                // Category 0: Move shifted register, ADD/SUB immediate
                if (opcode & 0xF800) == 0x0000
                    || (opcode & 0xF800) == 0x0800
                    || (opcode & 0xF800) == 0x1000
                    || (opcode & 0xF800) == 0x1800
                {
                    self.thumb_shift_register(opcode)
                } else {
                    self.thumb_add_sub_imm(opcode)
                }
            }
            0b001 => {
                // Category 1: ADD/SUB/CMP/MOV immediate
                self.thumb_data_proc_imm(opcode)
            }
            0b010 => {
                // Category 2: Data processing register
                let op = (opcode >> 6) & 0xF;
                if op <= 0x9 {
                    self.thumb_data_proc_reg(opcode)
                } else if (opcode & 0xFC00) == 0x4400 {
                    // Hi register operations / BX
                    self.thumb_hi_reg_ops(opcode)
                } else {
                    self.thumb_load_pc_rel(opcode, mem, instruction_pc)
                }
            }
            0b011 => {
                // Category 3: Load/store with offset
                let op = (opcode >> 11) & 0x3;
                match op {
                    0b00 => self.thumb_load_store_reg_offset(opcode, mem, false),
                    0b01 => self.thumb_load_store_reg_offset(opcode, mem, true),
                    0b10 => self.thumb_load_store_word_byte(opcode, mem, false),
                    0b11 => self.thumb_load_store_word_byte(opcode, mem, true),
                    _ => 1,
                }
            }
            0b100 => {
                // Category 4: Load/store sign-extended, halfword, sp-relative
                let op = (opcode >> 10) & 0x3;
                match op {
                    0b00 => self.thumb_load_store_halfword(opcode, mem, false),
                    0b01 => self.thumb_load_store_halfword(opcode, mem, true),
                    0b10 => self.thumb_load_store_sp_rel(opcode, mem, false),
                    0b11 => self.thumb_load_store_sp_rel(opcode, mem, true),
                    _ => 1,
                }
            }
            0b101 => {
                // Category 5: Load address, add offset to SP, push/pop
                if (opcode & 0xF800) == 0xA000 {
                    self.thumb_load_addr(opcode, instruction_pc)
                } else if (opcode & 0xFF00) == 0xB000 {
                    self.thumb_add_sp(opcode)
                } else if (opcode & 0xF600) == 0xB400 {
                    self.thumb_push_pop(opcode, mem, true)
                } else if (opcode & 0xF600) == 0xBC00 {
                    self.thumb_push_pop(opcode, mem, false)
                } else if (opcode & 0xF000) == 0xC000 {
                    self.thumb_load_store_multiple(opcode, mem, true)
                } else if (opcode & 0xF000) == 0xD000 {
                    self.thumb_load_store_multiple(opcode, mem, false)
                } else {
                    1
                }
            }
            0b110 => {
                // Category 6: Conditional branch, SWI, unconditional branch
                if (opcode & 0xF000) == 0xD000 {
                    self.thumb_branch_cond(opcode, instruction_pc)
                } else if (opcode & 0xFF00) == 0xDF00 {
                    self.thumb_software_interrupt(mem)
                } else {
                    self.thumb_branch(opcode, instruction_pc)
                }
            }
            0b111 => {
                // Category 7: Long branch with link (BL/BLX)
                let top5 = (opcode >> 11) & 0x1F;
                if top5 == 0b11110 || top5 == 0b11111 {
                    if (opcode & 0xF800) == 0xF000 {
                        self.thumb_bl_prefix(opcode, instruction_pc)
                    } else {
                        self.thumb_bl_suffix(opcode, instruction_pc)
                    }
                } else {
                    1
                }
            }
            _ => 1,
        }
    }

    // Thumb instruction implementations

    fn thumb_shift_register(&mut self, opcode: u16) -> u32 {
        let op = (opcode >> 11) & 0x3;
        let rm = ((opcode >> 3) & 0x7) as usize;
        let rd = (opcode & 0x7) as usize;
        let offset = ((opcode >> 6) & 0x1F) as u32;

        let mut result = self.r[rm];

        match op {
            0b00 => {
                // LSL
                let shift = offset.min(32);
                self.set_flag_c(if shift != 0 {
                    let bit = 32 - shift;
                    if bit <= 31 {
                        (result >> bit) & 1 != 0
                    } else {
                        false
                    }
                } else {
                    self.get_flag_c()
                });
                result = if shift < 32 { result << shift } else { 0 };
            }
            0b01 => {
                // LSR
                let shift = offset.min(32);
                self.set_flag_c(if shift != 0 {
                    (result >> (shift - 1)) & 1 != 0
                } else {
                    self.get_flag_c()
                });
                result = if shift < 32 { result >> shift } else { 0 };
            }
            0b10 => {
                // ASR
                let shift = offset.min(32);
                self.set_flag_c(if shift != 0 {
                    (result as i32 >> (shift - 1)) & 1 != 0
                } else {
                    self.get_flag_c()
                });
                result = ((result as i32) >> shift.min(31)) as u32;
            }
            _ => {}
        }

        self.r[rd] = result;
        self.set_flag_n((result as i32) < 0);
        self.set_flag_z(result == 0);
        self.r[15] = self.r[15].wrapping_add(2);
        1
    }

    fn thumb_add_sub_imm(&mut self, opcode: u16) -> u32 {
        let op = (opcode >> 9) & 0x3;
        let rn = ((opcode >> 6) & 0x7) as usize;
        let rd = (opcode & 0x7) as usize;
        let imm = ((opcode >> 3) & 0x7) as u32;

        let rn_val = self.r[rn];

        match op {
            0b00 => {
                // ADD Rd, Rn, #imm
                let (result, overflow) = rn_val.overflowing_add(imm);
                self.r[rd] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(overflow);
                self.set_flag_v(
                    ((rn_val as i32) > 0 && (imm as i32) > 0 && (result as i32) < 0)
                        || ((rn_val as i32) < 0 && (imm as i32) < 0 && (result as i32) > 0),
                );
            }
            0b01 => {
                // SUB Rd, Rn, #imm
                let (result, overflow) = rn_val.overflowing_sub(imm);
                self.r[rd] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(!overflow);
                self.set_flag_v(((rn_val as i32) < (imm as i32)) ^ ((result as i32) < 0));
            }
            0b10 => {
                // ADD Rd, Rn, #imm (with Rn = imm3:Rd)
                let rn_val = ((opcode >> 3) & 0x7 | (rd as u16 & 0x8)) as u32;
                let (result, overflow) = rn_val.overflowing_add(imm);
                self.r[rd] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(overflow);
            }
            0b11 => {
                // SUB Rd, Rn, #imm (with Rn = imm3:Rd)
                let rn_val = ((opcode >> 3) & 0x7 | (rd as u16 & 0x8)) as u32;
                let (result, overflow) = rn_val.overflowing_sub(imm);
                self.r[rd] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(!overflow);
            }
            _ => {}
        }

        self.r[15] = self.r[15].wrapping_add(2);
        1
    }

    fn thumb_data_proc_imm(&mut self, opcode: u16) -> u32 {
        let op = (opcode >> 11) & 0x3;
        let rd = ((opcode >> 8) & 0x7) as usize;
        let imm = (opcode & 0xFF) as u32;

        let rd_val = self.r[rd];

        match op {
            0b00 => {
                // MOV Rd, #imm
                self.r[rd] = imm;
                self.set_flag_n((imm as i32) < 0);
                self.set_flag_z(imm == 0);
            }
            0b01 => {
                // CMP Rd, #imm
                let (result, overflow) = rd_val.overflowing_sub(imm);
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(!overflow);
                self.set_flag_v(((rd_val as i32) < (imm as i32)) ^ ((result as i32) < 0));
            }
            0b10 => {
                // ADD Rd, #imm
                let (result, overflow) = rd_val.overflowing_add(imm);
                self.r[rd] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(overflow);
                self.set_flag_v(
                    ((rd_val as i32) > 0 && (imm as i32) > 0 && (result as i32) < 0)
                        || ((rd_val as i32) < 0 && (imm as i32) < 0 && (result as i32) > 0),
                );
            }
            0b11 => {
                // SUB Rd, #imm
                let (result, overflow) = rd_val.overflowing_sub(imm);
                self.r[rd] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(!overflow);
                self.set_flag_v(((rd_val as i32) < (imm as i32)) ^ ((result as i32) < 0));
            }
            _ => {}
        }

        self.r[15] = self.r[15].wrapping_add(2);
        1
    }

    fn thumb_data_proc_reg(&mut self, opcode: u16) -> u32 {
        let op = (opcode >> 6) & 0xF;
        let rms = ((opcode >> 3) & 0x7) as usize;
        let rds = (opcode & 0x7) as usize;

        let rm_val = self.r[rms];
        let rd_val = self.r[rds];

        match op {
            0x0 => {
                // AND Rd, Rm
                let result = rd_val & rm_val;
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            0x1 => {
                // EOR Rd, Rm
                let result = rd_val ^ rm_val;
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            0x2 => {
                // LSL Rd, Rm
                let shift = (rm_val & 0xFF).min(32);
                self.set_flag_c(if shift != 0 {
                    let bit = 32 - shift;
                    if bit <= 31 {
                        (rd_val >> bit) & 1 != 0
                    } else {
                        false
                    }
                } else {
                    self.get_flag_c()
                });
                let result = if shift < 32 { rd_val << shift } else { 0 };
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            0x3 => {
                // LSR Rd, Rm
                let shift = (rm_val & 0xFF).min(32);
                self.set_flag_c(if shift != 0 {
                    (rd_val >> (shift - 1)) & 1 != 0
                } else {
                    self.get_flag_c()
                });
                let result = if shift < 32 { rd_val >> shift } else { 0 };
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            0x4 => {
                // ASR Rd, Rm
                let shift = (rm_val & 0xFF).min(32);
                self.set_flag_c(if shift != 0 {
                    (rd_val as i32 >> (shift - 1)) & 1 != 0
                } else {
                    self.get_flag_c()
                });
                let result = ((rd_val as i32) >> shift.min(31)) as u32;
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            0x5 => {
                // ADC Rd, Rm
                let c = if self.get_flag_c() { 1 } else { 0 };
                let (result1, overflow1) = rd_val.overflowing_add(rm_val);
                let (result, overflow2) = result1.overflowing_add(c);
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(overflow1 || overflow2);
                let v = (!((rd_val ^ rm_val) >> 31 != 0)) && ((rd_val ^ result) >> 31 != 0);
                self.set_flag_v(v);
            }
            0x6 => {
                // SBC Rd, Rm
                let c = if self.get_flag_c() { 1 } else { 0 };
                let (result1, overflow1) = rd_val.overflowing_sub(rm_val);
                let (result, overflow2) = result1.overflowing_sub(c - 1);
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(overflow1 || overflow2);
            }
            0x7 => {
                // ROR Rd, Rm
                let shift = (rm_val & 0xFF) % 32;
                self.set_flag_c(if shift != 0 {
                    (rd_val >> (shift - 1)) & 1 != 0
                } else {
                    self.get_flag_c()
                });
                let result = rd_val.rotate_right(shift);
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            0x8 => {
                // TST Rd, Rm
                let result = rd_val & rm_val;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            0x9 => {
                // NEG Rd, Rm
                let (result, overflow) = 0u32.overflowing_sub(rm_val);
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(overflow);
                self.set_flag_v(rm_val == 0x80000000);
            }
            0xA => {
                // CMP Rd, Rm
                let (result, overflow) = rd_val.overflowing_sub(rm_val);
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(!overflow);
                self.set_flag_v(((rd_val as i32) < (rm_val as i32)) ^ ((result as i32) < 0));
            }
            0xB => {
                // CMN Rd, Rm
                let (result, overflow) = rd_val.overflowing_add(rm_val);
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(overflow);
            }
            0xC => {
                // ORR Rd, Rm
                let result = rd_val | rm_val;
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            0xD => {
                // MUL Rd, Rm
                let result = rd_val.wrapping_mul(rm_val);
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            0xE => {
                // BIC Rd, Rm
                let result = rd_val & !rm_val;
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            0xF => {
                // MVN Rd, Rm
                let result = !rm_val;
                self.r[rds] = result;
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
            }
            _ => {}
        }

        self.r[15] = self.r[15].wrapping_add(2);
        1
    }

    fn thumb_hi_reg_ops(&mut self, opcode: u16) -> u32 {
        let op = (opcode >> 8) & 0x3;
        let hd = ((opcode >> 7) & 1) != 0;
        let hsr = ((opcode >> 6) & 1) != 0;

        let rd = ((opcode & 0x7) | ((hd as u16) << 3)) as usize;
        let rs = (((opcode >> 3) & 0x7) | ((hsr as u16) << 3)) as usize;

        match op {
            0b00 => {
                // ADD
                let (result, overflow) = self.r[rd].overflowing_add(self.r[rs]);
                self.r[rd] = result;
                if !hd || !hsr {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(overflow);
                    self.set_flag_v(
                        ((self.r[rd] as i32) > 0 && (self.r[rs] as i32) > 0 && (result as i32) < 0)
                            || ((self.r[rd] as i32) < 0
                                && (self.r[rs] as i32) < 0
                                && (result as i32) > 0),
                    );
                }
            }
            0b01 => {
                // CMP
                let (result, overflow) = self.r[rd].overflowing_sub(self.r[rs]);
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(!overflow);
                self.set_flag_v(
                    ((self.r[rd] as i32) < (self.r[rs] as i32)) ^ ((result as i32) < 0),
                );
            }
            0b10 => {
                // MOV
                self.r[rd] = self.r[rs];
                if !hd || !hsr {
                    self.set_flag_n((self.r[rs] as i32) < 0);
                    self.set_flag_z(self.r[rs] == 0);
                }
            }
            0b11 => {
                // BX
                let target = self.r[rs];
                self.set_thumb_mode((target & 1) != 0);
                self.set_pc(target);
                return 2;
            }
            _ => {}
        }

        self.r[15] = self.r[15].wrapping_add(2);
        1
    }

    fn thumb_load_pc_rel(
        &mut self,
        opcode: u16,
        mem: &mut super::Memory,
        instruction_pc: u32,
    ) -> u32 {
        let rd = ((opcode >> 8) & 0x7) as usize;
        let imm = ((opcode & 0xFF) * 4) as u32;

        let pc = (instruction_pc + 4) & !0x3;
        let addr = pc.wrapping_add(imm);

        self.r[rd] = mem.read_word(addr);
        self.r[15] = self.r[15].wrapping_add(2);
        2
    }

    fn thumb_load_store_reg_offset(
        &mut self,
        opcode: u16,
        mem: &mut super::Memory,
        byte: bool,
    ) -> u32 {
        let ro = ((opcode >> 6) & 0x7) as usize;
        let rb = ((opcode >> 3) & 0x7) as usize;
        let rd = (opcode & 0x7) as usize;

        let addr = self.r[rb].wrapping_add(self.r[ro]);

        if byte {
            self.r[rd] = mem.read_byte(addr) as u32;
        } else {
            self.r[rd] = mem.read_word(addr);
        }

        self.r[15] = self.r[15].wrapping_add(2);
        2
    }

    fn thumb_load_store_word_byte(
        &mut self,
        opcode: u16,
        mem: &mut super::Memory,
        load: bool,
    ) -> u32 {
        let offset = (((opcode >> 6) & 0x1F) * 4) as u32;
        let rb = ((opcode >> 3) & 0x7) as usize;
        let rd = (opcode & 0x7) as usize;
        let byte = ((opcode >> 12) & 1) != 0;

        let addr = self.r[rb].wrapping_add(offset);

        if load {
            let val = if byte {
                mem.read_byte(addr) as u32
            } else {
                mem.read_word(addr)
            };

            if rd == 15 {
                // Loading into PC: branch to loaded address
                // ARM architecture: loaded value should have bit 0 cleared
                self.set_pc(val & 0xFFFFFFFE);
            } else {
                self.r[rd] = val;
            }
        } else {
            if byte {
                mem.write_byte(addr, self.r[rd] as u8);
            } else {
                mem.write_word(addr, self.r[rd]);
            }
        }

        self.r[15] = self.r[15].wrapping_add(2);
        2
    }

    fn thumb_load_store_halfword(
        &mut self,
        opcode: u16,
        mem: &mut super::Memory,
        load: bool,
    ) -> u32 {
        let offset = (((opcode >> 6) & 0x1F) * 2) as u32;
        let rb = ((opcode >> 3) & 0x7) as usize;
        let rd = (opcode & 0x7) as usize;

        let addr = self.r[rb].wrapping_add(offset);

        if load {
            self.r[rd] = mem.read_half(addr) as u32;
        } else {
            mem.write_half(addr, self.r[rd] as u16);
        }

        self.r[15] = self.r[15].wrapping_add(2);
        2
    }

    fn thumb_load_store_sp_rel(&mut self, opcode: u16, mem: &mut super::Memory, load: bool) -> u32 {
        let rd = ((opcode >> 8) & 0x7) as usize;
        let offset = ((opcode & 0xFF) * 4) as u32;

        let addr = self.r[13].wrapping_add(offset);

        if load {
            self.r[rd] = mem.read_word(addr);
        } else {
            mem.write_word(addr, self.r[rd]);
        }

        self.r[15] = self.r[15].wrapping_add(2);
        2
    }

    fn thumb_load_addr(&mut self, opcode: u16, instruction_pc: u32) -> u32 {
        let rd = ((opcode >> 8) & 0x7) as usize;
        let offset = ((opcode & 0xFF) * 4) as u32;

        let sp = ((opcode >> 11) & 1) != 0;

        let base = if sp {
            self.r[13]
        } else {
            (instruction_pc + 4) & !0x3
        };
        self.r[rd] = base.wrapping_add(offset);

        self.r[15] = self.r[15].wrapping_add(2);
        1
    }

    fn thumb_add_sp(&mut self, opcode: u16) -> u32 {
        let offset = (((opcode & 0x7F) * 4) as i32) as u32;
        let sign = ((opcode >> 7) & 1) != 0;

        if sign {
            self.r[13] = self.r[13].wrapping_sub(offset);
        } else {
            self.r[13] = self.r[13].wrapping_add(offset);
        }

        self.r[15] = self.r[15].wrapping_add(2);
        1
    }

    fn thumb_push_pop(&mut self, opcode: u16, mem: &mut super::Memory, load: bool) -> u32 {
        let pc_lr = ((opcode >> 8) & 1) != 0;
        let reg_list = opcode & 0xFF;

        let mut addr = self.r[13];

        if load {
            // POP (load from stack)
            for i in 0..8 {
                if reg_list & (1 << i) != 0 {
                    self.r[i] = mem.read_word(addr);
                    addr = addr.wrapping_add(4);
                }
            }
            if pc_lr {
                self.r[15] = mem.read_word(addr) & !1; // Return to ARM mode if bit 0 is 0
                addr = addr.wrapping_add(4);
            }
            self.r[13] = addr;
        } else {
            // PUSH (store to stack)
            if pc_lr {
                addr = addr.wrapping_sub(4);
                mem.write_word(addr, self.r[14]);
            }
            for i in (0..8).rev() {
                if reg_list & (1 << i) != 0 {
                    addr = addr.wrapping_sub(4);
                    mem.write_word(addr, self.r[i]);
                }
            }
            self.r[13] = addr;
        }

        self.r[15] = self.r[15].wrapping_add(2);
        (reg_list.count_ones() + if pc_lr { 1 } else { 0 }) as u32
    }

    fn thumb_load_store_multiple(
        &mut self,
        opcode: u16,
        mem: &mut super::Memory,
        load: bool,
    ) -> u32 {
        let rb = ((opcode >> 8) & 0x7) as usize;
        let reg_list = opcode & 0xFF;

        let mut addr = self.r[rb];

        if load {
            for i in 0..8 {
                if reg_list & (1 << i) != 0 {
                    self.r[i] = mem.read_word(addr);
                    addr = addr.wrapping_add(4);
                }
            }
            if reg_list & (1 << rb) == 0 {
                self.r[rb] = addr;
            }
        } else {
            for i in 0..8 {
                if reg_list & (1 << i) != 0 {
                    mem.write_word(addr, self.r[i]);
                    addr = addr.wrapping_add(4);
                }
            }
            if reg_list & (1 << rb) == 0 {
                self.r[rb] = addr;
            }
        }

        self.r[15] = self.r[15].wrapping_add(2);
        (reg_list.count_ones()) as u32
    }

    fn thumb_branch_cond(&mut self, opcode: u16, instruction_pc: u32) -> u32 {
        let cond = ((opcode >> 8) & 0xF) as usize;
        let offset = ((opcode as i8) as i32 * 2) as u32;

        if self.check_condition(cond) {
            let target = instruction_pc.wrapping_add(offset).wrapping_add(2);
            self.set_pc(target);
            return 1;
        }

        self.r[15] = self.r[15].wrapping_add(2);
        1
    }

    fn check_condition(&self, cond: usize) -> bool {
        match cond {
            0x0 => self.get_flag_z(),                                              // EQ
            0x1 => !self.get_flag_z(),                                             // NE
            0x2 => self.get_flag_c(),                                              // CS
            0x3 => !self.get_flag_c(),                                             // CC
            0x4 => self.get_flag_n(),                                              // MI
            0x5 => !self.get_flag_n(),                                             // PL
            0x6 => self.get_flag_v(),                                              // VS
            0x7 => !self.get_flag_v(),                                             // VC
            0x8 => self.get_flag_c() && !self.get_flag_z(),                        // HI
            0x9 => !(self.get_flag_c() && !self.get_flag_z()),                     // LO
            0xA => self.get_flag_n() == self.get_flag_v(),                         // GE
            0xB => self.get_flag_n() != self.get_flag_v(),                         // LT
            0xC => !self.get_flag_z() && (self.get_flag_n() == self.get_flag_v()), // GT
            0xD => self.get_flag_z() || (self.get_flag_n() != self.get_flag_v()),  // LE
            0xE => true,                                                           // AL
            _ => false,
        }
    }

    fn thumb_software_interrupt(&mut self, mem: &mut super::Memory) -> u32 {
        mem.set_bios_read_return(0xE3A02004);

        let swi_num = self.r[7];

        // Handle BIOS function calls
        match swi_num {
            0x00 => {
                // SoftReset
                self.reset();
                self.r[15] = 0x08000001; // Thumb mode
            }
            0x01 => {
                // RegisterRamReset
                self.r[15] = self.r[14] | 1; // Return in Thumb mode
            }
            0x02 | 0x03 => {
                // Halt / Stop
                self.r[15] = self.r[14] | 1;
            }
            0x04 => {
                // IntrWait
                self.r[15] = self.r[14] | 1;
            }
            0x05 => {
                // VBlankIntrWait
                self.r[15] = self.r[14] | 1;
            }
            0x06 => {
                // Div
                let r0 = self.r[0];
                let r1 = self.r[1];
                if r1 != 0 {
                    self.r[0] = r0 / r1;
                    self.r[3] = r0 % r1;
                } else {
                    self.r[0] = 0xFFFFFFFF;
                    self.r[3] = r0;
                }
                self.r[15] = self.r[14] | 1;
            }
            0x08 => {
                // DivArm
                let r0 = self.r[0];
                let r1 = self.r[1];
                if r1 != 0 {
                    self.r[0] = r0 / r1;
                    self.r[3] = r0 % r1;
                } else {
                    self.r[0] = 0xFFFFFFFF;
                    self.r[3] = r0;
                }
                self.r[15] = self.r[14] | 1;
            }
            0x0E => {
                // Sqrt
                let r0 = self.r[0] as f64;
                self.r[0] = (r0.sqrt()) as u32;
                self.r[15] = self.r[14] | 1;
            }
            _ => {
                // Unknown SWI
                if mem.has_bios() {
                    // Switch to Supervisor mode and jump to SWI vector
                    let old_cpsr = self.cpsr;
                    self.set_mode(Mode::Supervisor);
                    self.set_spsr(old_cpsr);
                    self.set_lr(self.r[15]);
                    self.r[15] = 0x08;
                    self.set_interrupts_enabled(false);
                } else {
                    self.r[15] = self.r[14] | 1;
                }
            }
        }

        2 + 2 // SWI takes 2 cycles + 2 for return
    }

    fn thumb_branch(&mut self, opcode: u16, instruction_pc: u32) -> u32 {
        let offset = ((opcode as i16) << 5) >> 4; // Sign-extend and multiply by 2
        let target = instruction_pc.wrapping_add(offset as u32).wrapping_add(2);
        self.set_pc(target);
        1
    }

    fn thumb_bl_prefix(&mut self, opcode: u16, _instruction_pc: u32) -> u32 {
        // BL/BLX prefix: 11110Sxxxxxxxxxxx
        // S: sign bit of offset (becomes bit 22 of final 23-bit offset)
        // imm10: bits 0-9 of offset (become bits 21-12 of final offset)
        let s_bit = (opcode >> 10) & 1;
        let imm10 = (opcode & 0x3FF) as u32;

        // Store partial offset in LR
        // We store s_bit at bit 22 (correct position for 23-bit offset)
        // and imm10 at bits 12-21 (already in correct position)
        let offset_high = ((s_bit as i32) << 22) | ((imm10 as i32) << 12);
        self.r[14] = offset_high as u32;

        // Advance to next instruction
        self.r[15] = self.r[15].wrapping_add(2);
        1
    }

    fn thumb_bl_suffix(&mut self, opcode: u16, _instruction_pc: u32) -> u32 {
        // BL/BLX suffix: 11H1Sxxxxxxxxxxx
        // H: determines if BL or BLX (1 = BLX, 0 = BL)
        // S: additional sign bit
        // imm11: bits 0-10 of offset (to be shifted left by 1)

        let h_bit = (opcode >> 12) & 1;
        let s_bit2 = (opcode >> 11) & 1;
        let imm11 = (opcode & 0x7FF) as u32;

        // Get the high offset from LR (stored by prefix)
        let offset_high = self.r[14] as i32;

        // Calculate full offset
        // BL offset encoding (23 bits total):
        // - Bit 22: s_bit (from prefix)
        // - Bits 21-12: imm10 (from prefix)
        // - Bit 11: s_bit2 (from suffix)
        // - Bits 10-0: imm11 << 1 (from suffix)
        // offset_high has: s_bit at bit 23 (to become bit 22), imm10 at bits 12-21
        // offset_low needs: s_bit2 at bit 11, imm11 at bits 0-10 (shifted left by 1)
        let offset_low = ((imm11 as i32) << 1) | ((s_bit2 as i32) << 11);
        let mut offset = offset_high.wrapping_add(offset_low);

        // Sign extend from 23 bits to 32 bits
        // The offset is a 23-bit signed value (bits 22-0)
        if (offset & 0x400000) != 0 {
            offset = offset | (-0x400000_i32);
        }

        // Save return address (PC of the next instruction after the suffix)
        self.r[14] = self.r[15].wrapping_add(2) | 1; // Return to Thumb mode

        // Calculate target
        let target = self.r[15].wrapping_add(2).wrapping_add(offset as u32);

        if h_bit == 1 {
            // BLX: switch to ARM mode (clear bit 0), target is word-aligned
            self.set_thumb_mode(false);
            self.set_pc(target & 0xFFFFFFFC); // Clear bit 0 for word alignment
        } else {
            // BL: stay in Thumb mode (set bit 0)
            self.set_pc(target | 1);
        }
        1
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cpu")
            .field("pc", &format!("{:#010X}", self.r[15]))
            .field("lr", &format!("{:#010X}", self.r[14]))
            .field("sp", &format!("{:#010X}", self.r[13]))
            .field("mode", &self.get_mode())
            .field("thumb", &self.is_thumb_mode())
            .finish()
    }
}
