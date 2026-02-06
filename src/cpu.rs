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
                // Load/store register offset / Media instructions
                self.execute_arm_load_store_register(opcode, mem)
            }
            0x3 => {
                // Branch / Branch with link
                self.execute_arm_branch(opcode, instruction_pc)
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
            0xD => {
                // MOV
                self.r[rd] = op2_val;
                if s {
                    self.set_flags_from_result(op2_val);
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
            _ => {
                // Other operations not yet implemented
                // TODO: Implement all ARM data processing instructions
            }
        }

        self.r[15] = self.r[15].wrapping_add(4);
        1 // 1 cycle for data processing
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

    fn execute_arm_psr(&mut self, _opcode: u32, _mem: &mut super::Memory) -> u32 {
        // MRS/MSR - not yet implemented
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

    fn execute_arm_load_store_register(&mut self, _opcode: u32, _mem: &mut super::Memory) -> u32 {
        // Not yet implemented
        1
    }

    fn execute_arm_branch(&mut self, opcode: u32, instruction_pc: u32) -> u32 {
        let offset = ((opcode as i32) << 8) >> 6; // Sign-extend and multiply by 4
        let link = ((opcode >> 24) & 1) != 0;

        // instruction_pc is the address of the instruction being executed
        // The branch offset is relative to this address
        if link {
            // Return address is the next instruction (instruction_pc + 4)
            self.set_lr(instruction_pc.wrapping_add(4));
        }

        // Calculate branch target
        let target = instruction_pc.wrapping_add(offset as u32);

        // Set PC using set_pc which aligns to word boundary
        self.set_pc(target);

        2 // Branch takes 2 cycles
    }

    fn step_thumb(&mut self, _mem: &mut super::Memory) -> u32 {
        // TODO: Implement Thumb instruction execution
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
