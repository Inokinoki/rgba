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
    pipeline: [u32; 3], // Prefetched instructions
    pipeline_pc: [u32; 3], // PC values for each prefetched instruction
    pipeline_loaded: bool,
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
        self.cpsr = 0x0000005F; // System mode, IRQ/FIQ enabled, ARM mode
        self.r[13] = 0x0300_7F00; // SP (stack pointer) - points to IWRAM
        self.r[14] = 0x0800_0000; // LR (link register) - points to ROM entry
        self.r[15] = 0x0800_0000; // PC (program counter) - ROM entry point
        self.pipeline = [0; 3];
        self.pipeline_pc = [0; 3];
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

    pub fn set_pc(&mut self, val: u32) {
        self.r[15] = val & 0xFFFFFFFC; // Align to word
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
        // Load pipeline if needed
        if !self.pipeline_loaded {
            self.pipeline_pc[0] = self.r[15];
            self.pipeline[0] = mem.read_word(self.r[15]);
            self.r[15] = self.r[15].wrapping_add(4);

            self.pipeline_pc[1] = self.r[15];
            self.pipeline[1] = mem.read_word(self.r[15]);
            self.r[15] = self.r[15].wrapping_add(4);

            self.pipeline_pc[2] = self.r[15];
            self.pipeline[2] = mem.read_word(self.r[15]);
            self.r[15] = self.r[15].wrapping_add(4);

            self.pipeline_loaded = true;
        }

        // Execute instruction, shift pipeline
        let opcode = self.pipeline[0];
        let instruction_pc = self.pipeline_pc[0];
        let pc_at_execution = self.r[15];

        self.pipeline[0] = self.pipeline[1];
        self.pipeline_pc[0] = self.pipeline_pc[1];

        self.pipeline[1] = self.pipeline[2];
        self.pipeline_pc[1] = self.pipeline_pc[2];

        // Decode and execute with instruction PC
        let cycles = self.execute_arm_with_pc(opcode, mem, instruction_pc, pc_at_execution);

        // Only fetch next instruction if PC wasn't modified (branch, etc.)
        if self.r[15] == pc_at_execution.wrapping_add(4) {
            // PC was incremented normally, fetch next
            self.pipeline_pc[2] = self.r[15];
            self.pipeline[2] = mem.read_word(self.r[15]);
            self.r[15] = self.r[15].wrapping_add(4);
        } else {
            // PC was modified by instruction (branch, etc.)
            // Pipeline will be reloaded on next call
            self.pipeline_loaded = false;
        }

        cycles
    }

    fn execute_arm_with_pc(&mut self, opcode: u32, mem: &mut super::Memory, instruction_pc: u32, pc_at_execution: u32) -> u32 {
        // ARM instruction decoding
        // Bits 27-26: Instruction category
        let category = (opcode >> 26) & 0x3;

        match category {
            0x0 => {
                // Data processing / PSR transfer
                if (opcode & 0x0FB0_0000) == 0x0100_0000 {
                    // PSR transfer
                    self.execute_arm_psr(opcode, mem)
                } else {
                    self.execute_arm_data_processing(opcode, mem)
                }
            }
            0x1 => {
                // Load/store immediate offset
                if (opcode & 0xFFFF_FFF0) == 0x0120_FFF0 {
                    // Branch and exchange
                    self.execute_arm_bx(opcode, mem)
                } else {
                    self.execute_arm_load_store(opcode, mem)
                }
            }
            0x2 => {
                // Load/store register offset / Branch
                // Branch instructions have bits 27-25 = 101
                if (opcode & 0x0E00_0000) == 0x0A00_0000 {
                    // Branch (B)
                    self.execute_arm_branch(opcode, instruction_pc, mem)
                } else {
                    self.execute_arm_load_store_register(opcode, mem)
                }
            }
            0x3 => {
                // Branch / Branch with link
                self.execute_arm_branch(opcode, instruction_pc, mem)
            }
            _ => 1, // Unknown, treat as NOP
        }
    }

    fn execute_arm_data_processing(&mut self, opcode: u32, _mem: &mut super::Memory) -> u32 {
        // Decode instruction
        let op = (opcode >> 21) & 0xF;
        let s = ((opcode >> 20) & 1) != 0;
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rd = ((opcode >> 12) & 0xF) as usize;
        let operand2 = opcode & 0xFFF;

        // Get operands
        let rn_val = self.r[rn];
        let op2_val = self.decode_operand2(operand2);

        match op {
            0x0 => {
                // AND
                let result = rn_val & op2_val;
                self.r[rd] = result;
                if s {
                    self.set_flags_from_result(result);
                }
            }
            0x1 => {
                // EOR
                let result = rn_val ^ op2_val;
                self.r[rd] = result;
                if s {
                    self.set_flags_from_result(result);
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
                    self.set_flag_v(((rn_val as i32) < (op2_val as i32)) ^
                                    ((result as i32) < 0));
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
                    self.set_flag_v(((op2_val as i32) < (rn_val as i32)) ^
                                    ((result as i32) < 0));
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
                    self.set_flag_v(((rn_val as i32) > 0 && (op2_val as i32) > 0 &&
                                    (result as i32) < 0) ||
                                   ((rn_val as i32) < 0 && (op2_val as i32) < 0 &&
                                    (result as i32) > 0));
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
                    self.set_flag_v(((rn_val as i32) > 0 && (op2_val as i32) > 0) ||
                                   ((result as i32) < 0));
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
                    self.set_flag_v(((rn_val as i32) < (borrow as i32)) ^
                                    ((result as i32) < 0));
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
                    self.set_flag_v(((op2_val as i32) < (borrow as i32)) ^
                                    ((result as i32) < 0));
                }
            }
            0x8 => {
                // TST
                let result = rn_val & op2_val;
                if s {
                    self.set_flags_from_result(result);
                }
            }
            0x9 => {
                // TEQ
                let result = rn_val ^ op2_val;
                if s {
                    self.set_flags_from_result(result);
                }
            }
            0xA => {
                // CMP
                let (result, overflow) = rn_val.overflowing_sub(op2_val);
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(!overflow);
                    self.set_flag_v(((rn_val as i32) < (op2_val as i32)) ^
                                    ((result as i32) < 0));
                }
            }
            0xB => {
                // CMN
                let (result, overflow) = rn_val.overflowing_add(op2_val);
                if s {
                    self.set_flag_n((result as i32) < 0);
                    self.set_flag_z(result == 0);
                    self.set_flag_c(overflow);
                    self.set_flag_v(((rn_val as i32) > 0 && (op2_val as i32) > 0 &&
                                    (result as i32) < 0) ||
                                   ((rn_val as i32) < 0 && (op2_val as i32) < 0 &&
                                    (result as i32) > 0));
                }
            }
            0xC => {
                // ORR
                let result = rn_val | op2_val;
                self.r[rd] = result;
                if s {
                    self.set_flags_from_result(result);
                }
            }
            0xD => {
                // MOV
                self.r[rd] = op2_val;
                if s {
                    self.set_flags_from_result(op2_val);
                }
            }
            0xE => {
                // BIC
                let result = rn_val & !op2_val;
                self.r[rd] = result;
                if s {
                    self.set_flags_from_result(result);
                }
            }
            0xF => {
                // MVN
                self.r[rd] = !op2_val;
                if s {
                    self.set_flags_from_result(!op2_val);
                }
            }
            _ => {}
        }

        self.r[15] = self.r[15].wrapping_add(4);
        1
    }

    fn decode_operand2(&self, operand2: u32) -> u32 {
        // Decode operand2 for data processing
        let imm = (operand2 & 1) != 0;
        let shift = (operand2 >> 4) & 0xFF;

        if imm {
            // Immediate value with rotate
            let imm8 = operand2 & 0xFF;
            let rotate = ((operand2 >> 8) & 0xF) * 2;
            imm8.rotate_right(rotate)
        } else {
            // Register with shift
            let rm = (operand2 & 0xF) as usize;
            let mut val = self.r[rm];

            let shift_type = (shift >> 1) & 0x3;
            let amount = shift >> 3;

            match shift_type {
                0 => val <<= amount, // LSL
                1 => val >>= amount, // LSR
                2 => val = ((val as i32) >> amount) as u32, // ASR
                3 => val = val.rotate_right(amount), // ROR
                _ => {}
            }

            val
        }
    }

    fn set_flags_from_result(&mut self, result: u32) {
        self.set_flag_n((result as i32) < 0);
        self.set_flag_z(result == 0);
        // C and V depend on the operation
    }

    fn execute_arm_psr(&mut self, opcode: u32, _mem: &mut super::Memory) -> u32 {
        let mrs = (opcode & (1 << 21)) != 0;
        let psr = (opcode & (1 << 22)) != 0; // 0 = CPSR, 1 = SPSR

        if mrs {
            // MRS - Transfer PSR to register
            let rd = ((opcode >> 12) & 0xF) as usize;
            if psr {
                // MRS Rd, SPSR_<mode>
                self.r[rd] = self.get_spsr();
            } else {
                // MRS Rd, CPSR
                self.r[rd] = self.cpsr;
            }
        } else {
            // MSR - Transfer register to PSR
            let rm = (opcode & 0xF) as usize;
            let immediate = (opcode & (1 << 25)) != 0;

            let val = if immediate {
                // Rotate immediate value
                let imm = opcode & 0xFF;
                let rotate = ((opcode >> 8) & 0xF) * 2;
                imm.rotate_right(rotate) as u32
            } else {
                self.r[rm]
            };

            let apply_flags = (opcode & 0x10000) != 0;
            let apply_control = (opcode & 0x20000) != 0;
            let apply_status = (opcode & 0x40000) != 0;
            let apply_extension = (opcode & 0x80000) != 0;

            if psr {
                // MSR SPSR_<mode>, {Rm|#imm}
                let mut spsr = self.get_spsr();
                if apply_flags {
                    spsr = (spsr & 0x0FFFFF00) | (val & 0x000000FF);
                }
                if apply_control || apply_status || apply_extension {
                    spsr = (spsr & 0x000000FF) | (val & 0xFFFFFF00);
                }
                self.set_spsr(spsr);
            } else {
                // MSR CPSR, {Rm|#imm}
                if apply_flags {
                    self.cpsr = (self.cpsr & 0x0FFFFF00) | (val & 0x000000FF);
                }
                if apply_control || apply_status || apply_extension {
                    self.cpsr = (self.cpsr & 0x000000FF) | (val & 0xFFFFFF00);
                    // Mode change might have happened
                    let new_mode = Mode::from_bits(self.cpsr);
                    if new_mode != self.get_mode() {
                        self.set_mode(new_mode);
                    }
                }
            }
        }

        self.r[15] = self.r[15].wrapping_add(4);
        1
    }

    fn execute_arm_bx(&mut self, opcode: u32, _mem: &mut super::Memory) -> u32 {
        let rm = (opcode & 0xF) as usize;
        let target = self.r[rm];

        self.set_thumb_mode((target & 1) != 0);
        self.set_pc(target);

        2 // BX takes 2 cycles
    }

    fn execute_arm_load_store(&mut self, _opcode: u32, mem: &mut super::Memory) -> u32 {
        // LDR/STR - simplified implementation
        let rn = ((_opcode >> 16) & 0xF) as usize;
        let rd = ((_opcode >> 12) & 0xF) as usize;
        let offset = (_opcode & 0xFFF) as i32 as i64;
        let base = self.r[rn] as i64;

        let load = (_opcode >> 20) & 1 != 0;
        let byte = (_opcode >> 22) & 1 != 0;
        let add = (_opcode >> 23) & 1 != 0;
        let pre_index = (_opcode >> 24) & 1 != 0;

        let addr = if add {
            (base + offset) as u32
        } else {
            (base - offset) as u32
        };

        if load {
            if byte {
                self.r[rd] = mem.read_byte(addr) as u32;
            } else {
                self.r[rd] = mem.read_word(addr);
            }
        } else {
            if byte {
                mem.write_byte(addr, self.r[rd] as u8);
            } else {
                mem.write_word(addr, self.r[rd]);
            }
        }

        if !pre_index {
            // Post-index: update base register
            self.r[rn] = addr;
        }

        self.r[15] = self.r[15].wrapping_add(4);
        2 // Memory access takes at least 2 cycles
    }

    fn execute_arm_load_store_register(&mut self, opcode: u32, mem: &mut super::Memory) -> u32 {
        let load = ((opcode >> 20) & 1) != 0;
        let byte = ((opcode >> 22) & 1) != 0;
        let writeback = ((opcode >> 21) & 1) != 0;
        let pre_index = ((opcode >> 24) & 1) != 0;
        let add = ((opcode >> 23) & 1) != 0;
        let rn = ((opcode >> 16) & 0xF) as usize;
        let rd = ((opcode >> 12) & 0xF) as usize;
        let rm = (opcode & 0xF) as usize;

        let shift_type = (opcode >> 5) & 0x3;
        let shift_amount = ((opcode >> 7) & 0x1F) as u32;

        let mut offset = self.r[rm];
        match shift_type {
            0 => offset <<= shift_amount, // LSL
            1 => offset >>= shift_amount, // LSR
            2 => offset = ((offset as i32) >> shift_amount) as u32, // ASR
            3 => offset = offset.rotate_right(shift_amount), // ROR
            _ => {}
        }

        let base = self.r[rn] as i64;
        let addr = if add {
            (base + offset as i64) as u32
        } else {
            (base - offset as i64) as u32
        };

        if load {
            if byte {
                self.r[rd] = mem.read_byte(addr) as u32;
            } else {
                self.r[rd] = mem.read_word(addr);
            }
        } else {
            if byte {
                mem.write_byte(addr, self.r[rd] as u8);
            } else {
                mem.write_word(addr, self.r[rd]);
            }
        }

        if writeback {
            if pre_index {
                self.r[rn] = addr;
            } else {
                self.r[rn] = addr;
            }
        }

        self.r[15] = self.r[15].wrapping_add(4);
        2
    }

    fn execute_arm_branch(&mut self, opcode: u32, instruction_pc: u32, mem: &mut super::Memory) -> u32 {
        // Check for SWI (Software Interrupt)
        // ARM SWI pattern: bits 27-26 = 0b11, bit 24 = 0, bits 23-8 = 0xFFFF
        if (opcode & 0x0F00_0000) == 0x0F00_0000 {
            return self.execute_arm_swi(mem);
        }

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

        // instruction_pc is the address of the instruction being executed
        // The branch offset is relative to this address
        if link {
            // Return address is the next instruction (instruction_pc + 4)
            self.set_lr(instruction_pc.wrapping_add(4));
        }

        // Calculate branch target
        let target = instruction_pc.wrapping_add(8).wrapping_add(offset as u32);

        // Set PC using set_pc which aligns to word boundary
        self.set_pc(target);

        2 // Branch takes 2 cycles
    }

    fn execute_arm_swi(&mut self, mem: &mut super::Memory) -> u32 {
        // Extract SWI function number from instruction (bits 23-0 for ARM)
        // Note: We need to read the instruction that caused this SWI
        // The instruction is at LR - 4 (since LR was set to next instruction)
        let swi_insn = mem.read_word(self.r[14] - 4);
        let swi_func = swi_insn & 0xFFFFFF;

        // For Thumb mode, SWI number is in R7 (but we handle Thumb SWI separately)

        // Handle BIOS function calls
        match swi_func {
            0x00 => {
                // SoftReset - reset the system
                self.reset();
                self.r[15] = 0x08000000;
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
                // Div - division (not implemented in real BIOS)
                // For now, return R0 / R1 in R0 and R3
                let r0 = self.r[0];
                let r1 = self.r[1];
                if r1 != 0 {
                    self.r[0] = r0 / r1;
                    self.r[3] = r0 % r1;
                } else {
                    self.r[0] = 0xFFFFFFFF;
                    self.r[3] = r0;
                }
                self.r[15] = self.r[14];
            }
            0x08 => {
                // DivArm - same as Div
                let r0 = self.r[0];
                let r1 = self.r[1];
                if r1 != 0 {
                    self.r[0] = r0 / r1;
                    self.r[3] = r0 % r1;
                } else {
                    self.r[0] = 0xFFFFFFFF;
                    self.r[3] = r0;
                }
                self.r[15] = self.r[14];
            }
            0x0E => {
                // Sqrt - square root
                let r0 = self.r[0] as f64;
                self.r[0] = (r0.sqrt()) as u32;
                self.r[15] = self.r[14];
            }
            _ => {
                // Unknown SWI - try jumping to BIOS if available
                if mem.has_bios() {
                    // Switch to Supervisor mode and jump to SWI vector (0x08)
                    let old_cpsr = self.cpsr;
                    self.set_mode(Mode::Supervisor);
                    self.set_spsr(old_cpsr);
                    self.set_lr(self.r[15]); // Return address is next instruction
                    self.r[15] = 0x08; // SWI vector
                    self.set_interrupts_enabled(false);
                } else {
                    // No BIOS - just return
                    eprintln!("Warning: Unknown SWI 0x{:06X}, returning without action", swi_func);
                    self.r[15] = self.r[14];
                }
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

        self.pipeline[0] = self.pipeline[1];
        self.pipeline_pc[0] = self.pipeline_pc[1];

        self.pipeline[1] = self.pipeline[2];
        self.pipeline_pc[1] = self.pipeline_pc[2];

        // Decode and execute
        let cycles = self.execute_thumb(opcode, mem, instruction_pc);

        // Only fetch next instruction if PC wasn't modified
        if self.r[15] == pc_at_execution.wrapping_add(2) {
            self.pipeline_pc[2] = self.r[15];
            self.pipeline[2] = mem.read_half(self.r[15]) as u32;
            self.r[15] = self.r[15].wrapping_add(2);
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
                if (opcode & 0xF800) == 0x0000 || (opcode & 0xF800) == 0x0800 ||
                   (opcode & 0xF800) == 0x1000 || (opcode & 0xF800) == 0x1800 {
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
                    self.thumb_load_pc_rel(opcode, mem)
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
                    _ => 1
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
                    _ => 1
                }
            }
            0b101 => {
                // Category 5: Load address, add offset to SP, push/pop
                if (opcode & 0xF800) == 0xA000 {
                    self.thumb_load_addr(opcode)
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
                if (opcode & 0xF800) == 0xE000 {
                    self.thumb_branch_cond(opcode, instruction_pc)
                } else if (opcode & 0xFF00) == 0xDF00 {
                    self.thumb_software_interrupt(mem)
                } else {
                    self.thumb_branch(opcode, instruction_pc)
                }
            }
            0b111 => {
                // Category 7: Long branch with link
                if (opcode & 0xF800) == 0xF000 {
                    self.thumb_bl_prefix(opcode, instruction_pc)
                } else {
                    self.thumb_bl_suffix(opcode, instruction_pc)
                }
            }
            _ => 1
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
                    if bit <= 31 { (result >> bit) & 1 != 0 } else { false }
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
                self.set_flag_v(((rn_val as i32) > 0 && (imm as i32) > 0 && (result as i32) < 0) ||
                               ((rn_val as i32) < 0 && (imm as i32) < 0 && (result as i32) > 0));
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
                self.set_flag_v(((rd_val as i32) > 0 && (imm as i32) > 0 && (result as i32) < 0) ||
                               ((rd_val as i32) < 0 && (imm as i32) < 0 && (result as i32) > 0));
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
                    if bit <= 31 { (rd_val >> bit) & 1 != 0 } else { false }
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
                    self.set_flag_v(((self.r[rd] as i32) > 0 && (self.r[rs] as i32) > 0 && (result as i32) < 0) ||
                                   ((self.r[rd] as i32) < 0 && (self.r[rs] as i32) < 0 && (result as i32) > 0));
                }
            }
            0b01 => {
                // CMP
                let (result, overflow) = self.r[rd].overflowing_sub(self.r[rs]);
                self.set_flag_n((result as i32) < 0);
                self.set_flag_z(result == 0);
                self.set_flag_c(!overflow);
                self.set_flag_v(((self.r[rd] as i32) < (self.r[rs] as i32)) ^ ((result as i32) < 0));
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

    fn thumb_load_pc_rel(&mut self, opcode: u16, mem: &mut super::Memory) -> u32 {
        let rd = ((opcode >> 8) & 0x7) as usize;
        let imm = ((opcode & 0xFF) * 4) as u32;

        let pc = self.r[15] & !0x3; // Align to word
        let addr = pc.wrapping_add(imm);

        self.r[rd] = mem.read_word(addr);
        self.r[15] = self.r[15].wrapping_add(2);
        2
    }

    fn thumb_load_store_reg_offset(&mut self, opcode: u16, mem: &mut super::Memory, byte: bool) -> u32 {
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

    fn thumb_load_store_word_byte(&mut self, opcode: u16, mem: &mut super::Memory, load: bool) -> u32 {
        let offset = (((opcode >> 6) & 0x1F) * 4) as u32;
        let rb = ((opcode >> 3) & 0x7) as usize;
        let rd = (opcode & 0x7) as usize;
        let byte = ((opcode >> 12) & 1) != 0;

        let addr = self.r[rb].wrapping_add(offset);

        if load {
            if byte {
                self.r[rd] = mem.read_byte(addr) as u32;
            } else {
                self.r[rd] = mem.read_word(addr);
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

    fn thumb_load_store_halfword(&mut self, opcode: u16, mem: &mut super::Memory, load: bool) -> u32 {
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

    fn thumb_load_addr(&mut self, opcode: u16) -> u32 {
        let rd = ((opcode >> 8) & 0x7) as usize;
        let offset = ((opcode & 0xFF) * 4) as u32;

        let sp = ((opcode >> 11) & 1) != 0;

        let base = if sp { self.r[13] } else { self.r[15] & !0x3 };
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

    fn thumb_load_store_multiple(&mut self, opcode: u16, mem: &mut super::Memory, load: bool) -> u32 {
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
            0x0 => self.get_flag_z(),        // EQ
            0x1 => !self.get_flag_z(),       // NE
            0x2 => self.get_flag_c(),        // CS
            0x3 => !self.get_flag_c(),       // CC
            0x4 => self.get_flag_n(),        // MI
            0x5 => !self.get_flag_n(),       // PL
            0x6 => self.get_flag_v(),        // VS
            0x7 => !self.get_flag_v(),       // VC
            0x8 => self.get_flag_c() && !self.get_flag_z(),  // HI
            0x9 => !(self.get_flag_c() && !self.get_flag_z()), // LO
            0xA => self.get_flag_n() == self.get_flag_v(),    // GE
            0xB => self.get_flag_n() != self.get_flag_v(),    // LT
            0xC => !self.get_flag_z() && (self.get_flag_n() == self.get_flag_v()), // GT
            0xD => self.get_flag_z() || (self.get_flag_n() != self.get_flag_v()), // LE
            0xE => true,                     // AL
            _ => false,
        }
    }

    fn thumb_software_interrupt(&mut self, mem: &mut super::Memory) -> u32 {
        // Thumb SWI: function number is in R7
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
                    eprintln!("Warning: Unknown Thumb SWI 0x{:02X}, returning without action", swi_num);
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

    fn thumb_bl_prefix(&mut self, opcode: u16, instruction_pc: u32) -> u32 {
        let offset = ((opcode as i16) << 5) >> 4; // Sign-extend upper 11 bits
        let target = instruction_pc.wrapping_add((offset as u32) << 11);
        self.r[14] = target | 1; // Set bit 0 to indicate Thumb
        self.r[15] = self.r[15].wrapping_add(2);
        1
    }

    fn thumb_bl_suffix(&mut self, opcode: u16, _instruction_pc: u32) -> u32 {
        let offset = ((opcode & 0x7FF) as u32) * 2;
        let target = (self.r[14] & !1).wrapping_add(offset);
        self.r[14] = self.r[15].wrapping_sub(2) | 1;
        self.set_pc(target);
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
