//! GBA Memory System
//!
//! The GBA has a complex memory map with different regions having different
//! access timings and characteristics.
//!
//! Memory Map:
//! - 0x0000_0000 - 0x0000_3FFF: BIOS (16KB)
//! - 0x0200_0000 - 0x0203_FFFF: WRAM-B (256KB)
//! - 0x0300_0000 - 0x0300_7FFF: IWRAM (32KB)
//! - 0x0400_0000 - 0x0400_03FE: IO Registers
//! - 0x0500_0000 - 0x0500_03FF: Palette RAM (1KB)
//! - 0x0600_0000 - 0x0601_7FFF: VRAM (96KB)
//! - 0x0700_0000 - 0x0700_03FF: OAM (1KB)
//! - 0x0800_0000 - 0x0DFF_FFFF: ROM (max 32MB)

use bitflags::bitflags;

use crate::{Eeprom, Flash};

/// Cartridge save type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveType {
    None,
    Sram,
    Flash64K,
    Flash128K,
    Eeprom512B,
    Eeprom8K,
}

bitflags! {
    /// Interrupt flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Interrupt: u16 {
        const VBLANK   = 0x0001;
        const HBLANK   = 0x0002;
        const VCOUNT   = 0x0004;
        const TIMER0   = 0x0008;
        const TIMER1   = 0x0010;
        const TIMER2   = 0x0020;
        const TIMER3   = 0x0040;
        const SERIAL   = 0x0080;
        const DMA0     = 0x0100;
        const DMA1     = 0x0200;
        const DMA2     = 0x0400;
        const DMA3     = 0x0800;
        const KEYPAD   = 0x1000;
        const GAMEPAK  = 0x2000;
    }
}

/// GBA Interrupt Controller (embedded in Memory for IO register handling)
pub struct InterruptController {
    /// Interrupt Enable register (0x0400_0200)
    pub ie: Interrupt,

    /// Interrupt Enable/FIRQ select (0x0400_0208)
    pub ie_fp: Interrupt,

    /// Interrupt Request flags (0x0400_0202)
    pub if_raw: Interrupt,

    /// Interrupt flags with masked disabled interrupts cleared
    if_processed: Interrupt,

    /// Interrupt Master Enable (0x0400_0208)
    pub ime: bool,

    /// Whether we're currently in an interrupt handler
    pub in_interrupt: bool,
}

impl InterruptController {
    pub fn new() -> Self {
        Self {
            ie: Interrupt::empty(),
            ie_fp: Interrupt::empty(),
            if_raw: Interrupt::empty(),
            if_processed: Interrupt::empty(),
            ime: false,
            in_interrupt: false,
        }
    }

    pub fn reset(&mut self) {
        self.ie = Interrupt::empty();
        self.ie_fp = Interrupt::empty();
        self.if_raw = Interrupt::empty();
        self.if_processed = Interrupt::empty();
        self.ime = false;
        self.in_interrupt = false;
    }

    /// Request an interrupt
    pub fn request(&mut self, interrupt: Interrupt) {
        self.if_raw |= interrupt;
    }

    /// Get pending interrupt (considering IE and IME)
    pub fn get_pending(&self) -> Option<Interrupt> {
        if !self.ime {
            return None;
        }

        // Get enabled interrupts that have requested
        let pending = self.ie & self.if_raw;

        if pending.is_empty() {
            None
        } else {
            // Return the highest priority interrupt (lowest bit number)
            let bit = pending.bits().trailing_zeros() as u16;
            Some(Interrupt::from_bits_truncate(1 << bit))
        }
    }

    /// Acknowledge an interrupt (clears IF bit)
    pub fn acknowledge(&mut self, interrupt: Interrupt) {
        self.if_raw &= !interrupt;
        self.if_processed &= !interrupt;
    }

    /// Check if we should take an interrupt (IME must be set)
    pub fn should_take_interrupt(&self) -> bool {
        if !self.ime || self.in_interrupt {
            return false;
        }

        !(self.ie & self.if_raw).is_empty()
    }

    /// Check if HALT condition is met (IF & IE != 0, regardless of IME)
    /// The GBA CPU wakes from HALT when any enabled interrupt is pending,
    /// even if IME is 0 (interrupts disabled). IME only controls whether
    /// the CPU actually enters the IRQ handler.
    pub fn should_wake_from_halt(&self) -> bool {
        !(self.ie & self.if_raw).is_empty()
    }

    /// Enter interrupt handler
    pub fn enter_interrupt(&mut self) {
        self.in_interrupt = true;
        self.ime = false; // IME is cleared on interrupt entry
    }

    /// Exit interrupt handler
    pub fn exit_interrupt(&mut self) {
        self.in_interrupt = false;
        self.ime = true; // IME is restored on interrupt exit
    }

    /// Read IO register
    pub fn read_register(&self, offset: usize) -> u16 {
        match offset {
            0x000 => self.ie.bits(),
            0x002 => self.if_raw.bits(),
            0x200 => self.ie.bits(),
            0x208 => self.ime as u16,
            _ => 0,
        }
    }

    /// Write IO register
    pub fn write_register(&mut self, offset: usize, val: u16) {
        match offset {
            0x000 => self.ie = Interrupt::from_bits_truncate(val),
            0x002 => {
                // IF - writing 1 clears the bit, writing 0 has no effect
                self.if_raw &= !(Interrupt::from_bits_truncate(val));
                self.if_processed &= !(Interrupt::from_bits_truncate(val));
            }
            0x200 => self.ie = Interrupt::from_bits_truncate(val),
            0x208 => self.ime = val != 0,
            _ => {}
        }
    }
}

impl Default for InterruptController {
    fn default() -> Self {
        Self::new()
    }
}

/// GBA Memory System
pub struct Memory {
    // BIOS ROM (16KB) - read-only after boot
    bios: Vec<u8>,

    // BIOS read return value (for addresses 0-3)
    // On real GBA, reading from BIOS returns special values based on BIOS state
    bios_read_return: u32,

    pub use_real_bios: bool,
    pub intrwait_flag_addr: u32,
    pub intrwait_active: bool,

    // On-board Work RAM (256KB) - 3 cycles
    wram: Box<[u8; 0x40000]>,

    // On-chip Work RAM (32KB) - 1 cycle (fastest!)
    iwram: Box<[u8; 0x8000]>,

    // IO Registers (1KB)
    io: Box<[u8; 0x400]>,

    // Snapshot of IO registers at VBlank start (for correct rendering)
    pub vblank_io_snapshot: Option<Vec<u8>>,

    // Palette RAM (1KB) - 2KB actually, split BG/OBJ
    palette: Box<[u8; 0x400]>,

    // Video RAM (96KB) - can be accessed as 16-bit or 32-bit
    vram: Box<[u8; 0x18000]>,

    // Object Attribute Memory (1KB)
    oam: Box<[u8; 0x400]>,

    // SRAM (32KB) - cartridge battery-backed RAM
    sram: Box<[u8; 0x8000]>,

    // ROM (max 32MB) - mirrored across different waitstate regions
    rom: Vec<u8>,

    // Waitstate configuration
    waitcnt: u16,

    // Interrupt controller
    pub interrupt: InterruptController,

    // HALT state - set when writing to HALTCNT (0x0400_0301)
    pub halt_pending: bool,

    // Dirty flags for lazy synchronization
    pub vram_dirty: bool,
    pub oam_dirty: bool,
    pub palette_dirty: bool,
    pub io_ppu_dirty: bool,
    pub io_timer_dirty: bool,
    pub io_dma_dirty: bool,
    pub dma_active: bool,

    // Save type configuration and backends
    save_type: SaveType,
    flash: Option<Flash>,
    eeprom: Option<Eeprom>,

    pub ewram_write_limit: Option<u32>,

    pub vram_write_log: Vec<(u32, u32, u8)>,
    pub vram_log_enabled: bool,
    pub vram_log_pc: u32,
    pub ewram_tile_log: Vec<(u32, u32, u8)>,
    pub ewram_tile_log_enabled: bool,
    pub palette_write_log: Vec<(u32, u32, u8, bool)>,
    pub palette_log_enabled: bool,
    pub dma_log: Vec<(u8, u32, u32, u32, u32)>,
    pub dma_log_enabled: bool,
    pub swi_log: Vec<u32>,
    pub swi_log_enabled: bool,
    pub arm_swi_count: u32,
    pub thumb_swi_count: u32,
    pub cpu_set_log: Vec<(u32, u32, u32)>,
    pub cpu_set_log_enabled: bool,
    pub pc_trace_base: u32,
    pub pc_trace_counts: Vec<u32>,
    pub irq_trace: Vec<(u32, u32, u16, u16, bool)>,
    pub irq_trace_enabled: bool,
    pub iwram_write_log: Vec<(u32, u32, u8)>,
    pub iwram_write_log_enabled: bool,
    pub ewram_range_log: Vec<(u32, u32, u8)>,
    pub ewram_range_log_enabled: bool,
    pub reg_snapshots: Vec<[u32; 16]>,
    pub reg_snapshot_enabled: bool,
    pub decomp_writes: Vec<(u32, u32, u8)>,
    pub decomp_writes_enabled: bool,
    pub keyinput_read_trace_enabled: bool,
    pub keyinput_read_pcs: Vec<u32>,
    pub keyinput_read_trace_pc: u32,
    pub timer_writes: Vec<(u32, u32, u8)>,
    pub timer_writes_enabled: bool,
    pub input_reads: Vec<(u32, u32)>,
    pub input_reads_enabled: bool,
    pub dispcnt_write_log: Vec<(u32, u8, u8)>,
    pub dispcnt_write_log_enabled: bool,
    pub ie_ime_write_log: Vec<(u32, u32, u16)>, // (pc, addr, val)
    pub ie_ime_write_log_enabled: bool,
}

impl Memory {
    pub fn new() -> Self {
        // Initialize BIOS with stub implementation
        // Fill with BX LR (0xE12FFF1E) - all bytes non-zero, so games
        // that read BIOS bytes as data won't get false zeros.
        // BX LR is safe because if code falls through, it just returns.
        let mut bios = vec![0u8; 0x4000];

        for i in (0..0x4000).step_by(4) {
            bios[i] = 0x1E;
            bios[i + 1] = 0xFF;
            bios[i + 2] = 0x2F;
            bios[i + 3] = 0xE1;
        }

        // At BIOS entry point (0x00000000), jump to ROM at 0x08000000
        // LDR PC, [PC, #4] = 0xE59FF004 -> loads from 0x00+8+4=0x0C
        // Then at 0x0C: .word 0x08000000
        bios[0] = 0x04;
        bios[1] = 0xF0;
        bios[2] = 0x9F;
        bios[3] = 0xE5;
        // 0x04-0x0B: filler (already BX LR from the loop)
        // 0x0C-0x0F: target address
        bios[0x0C] = 0x00;
        bios[0x0D] = 0x00;
        bios[0x0E] = 0x00;
        bios[0x0F] = 0x08;

        // At key BIOS entry points used by tests and games, put "BX LR" to return
        // BX LR in ARM: 0xE12FFF1E
        let bios_return: [u8; 4] = [0x1E, 0xFF, 0x2F, 0xE1];

        // Set returns at common BIOS call points
        // These include addresses used by gba-tests and common BIOS function entry points
        // that commercial games may call directly via BX (not via SWI)
        for offset in [
            0x0C4, 0x0DC, 0x128, 0x130, 0x138, 0x140, 0x148, 0x150, 0x158, 0x164, 0x17C, 0x188,
            0x190, 0x198, 0x1A0, 0x1A8, 0x1B0, 0x1B8, 0x1C0, 0x1C8, 0x1D0, 0x1D8, 0x1E0, 0x1E8,
            0x1F0, 0x1F8, 0x200, 0x208, 0x210, 0x218, 0x220, 0x228, 0x230, 0x238, 0x240, 0x248,
            0x250, 0x258, 0x260, 0x268, 0x270, 0x278, 0x280, 0x288, 0x290, 0x298, 0x2A0, 0x2A8,
            0x2B0, 0x2B8, 0x2C0, 0x2C8, 0x2D0, 0x2D8, 0x2E0, 0x2E8, 0x2F0, 0x2F8, 0x300, 0x400,
            0x500, 0x600, 0x700, 0x800, 0x900, 0xA00, 0xB00, 0xC00, 0xD00, 0xE00, 0xF00, 0x1000,
            0x1100, 0x1200, 0x1300, 0x1400, 0x1500, 0x1600, 0x1700, 0x1800, 0x1900, 0x1A00, 0x1B00,
            0x1C00, 0x1D00, 0x1E00, 0x1F00, 0x2000, 0x2100, 0x2200, 0x2300, 0x2400, 0x2500, 0x2600,
            0x2700, 0x2800, 0x2900, 0x2A00, 0x2B00, 0x2C00, 0x2D00, 0x2E00, 0x2F00, 0x3000, 0x3100,
            0x3200, 0x3300, 0x3400, 0x3500, 0x3600, 0x3700, 0x3800, 0x3900, 0x3A00, 0x3B00, 0x3C00,
            0x3D00, 0x3E00, 0x3F00,
        ]
        .iter()
        {
            if *offset + 4 <= 0x4000 {
                bios[*offset..(*offset + 4)].copy_from_slice(&bios_return);
            }
        }

        // Place BIOS stub handler at 0x013C that clears IF and returns
        // On real GBA, the user handler clears IF. This stub handles the case
        // where the game hasn't installed its handler yet.
        // ARM code at 0x013C:
        //   LDR R0, [PC, #8]       ; load IF register address from literal pool
        //   MOV R1, #1             ; VBlank bit
        //   STRH R1, [R0]          ; clear VBlank in IF
        //   BX LR                  ; return to BIOS dispatcher (not exception return)
        //   .word 0x04000202       ; literal pool: IF register address
        bios[0x013C] = 0x08;
        bios[0x013D] = 0x00;
        bios[0x013E] = 0x9F;
        bios[0x013F] = 0xE5; // 0x013C: LDR R0, [PC, #8] -> loads from 0x014C
        bios[0x0140] = 0x01;
        bios[0x0141] = 0x10;
        bios[0x0142] = 0xA0;
        bios[0x0143] = 0xE3; // 0x0140: MOV R1, #1
        bios[0x0144] = 0xB0;
        bios[0x0145] = 0x10;
        bios[0x0146] = 0xC0;
        bios[0x0147] = 0xE1; // 0x0144: STRH R1, [R0]
        bios[0x0148] = 0x1E;
        bios[0x0149] = 0xFF;
        bios[0x014A] = 0x2F;
        bios[0x014B] = 0xE1; // 0x0148: BX LR
        bios[0x014C] = 0x02;
        bios[0x014D] = 0x02;
        bios[0x014E] = 0x00;
        bios[0x014F] = 0x04; // 0x014C: .word 0x04000202

        // BIOS IRQ dispatcher at 0x0018 (ARM IRQ vector)
        // Calls the user handler at [0x03007FFC].
        // Saves/restores R0-R3, R12, LR across handler call.
        let irq_handler: [u8; 60] = [
            0x1F, 0x50, 0x2D, 0xE9, // 0x0018: STMFD SP!, {R0-R3, R12, LR}
            0x00, 0x10, 0x4F, 0xE1, // 0x001C: MRS R1, SPSR
            0x02, 0x00, 0x2D, 0xE9, // 0x0020: STMFD SP!, {R1}
            0x24, 0x00, 0x9F, 0xE5, // 0x0024: LDR R0, [PC, #0x24] -> loads from 0x0050
            0x00, 0x00, 0x90, 0xE5, // 0x0028: LDR R0, [R0]
            0x00, 0x00, 0x50, 0xE3, // 0x002C: CMP R0, #0
            0x02, 0x00, 0x00, 0x0A, // 0x0030: BEQ restore_spsr (-> 0x0040)
            0x0F, 0xE0, 0xA0, 0xE1, // 0x0034: MOV LR, PC
            0x04, 0xE0, 0x8E, 0xE2, // 0x0038: ADD LR, LR, #4
            0x10, 0xFF, 0x2F, 0xE1, // 0x003C: BX R0
            // restore_spsr:
            0x02, 0x00, 0xBD, 0xE8, // 0x0040: LDMFD SP!, {R1}
            0x01, 0x90, 0x6F, 0xE1, // 0x0044: MSR SPSR_fc, R1
            0x1F, 0x50, 0xBD, 0xE8, // 0x0048: LDMFD SP!, {R0-R3, R12, LR}
            0x04, 0xF0, 0x5E, 0xE2, // 0x004C: SUBS PC, LR, #4
            0xFC, 0x7F, 0x00, 0x03, // 0x0050: .word 0x03007FFC
        ];
        bios[0x18..0x18 + irq_handler.len()].copy_from_slice(&irq_handler);

        // Set IRQ handler pointer at 0x03007FFC to point to BIOS stub
        // Game initialization will overwrite this when it sets up its own handler.
        let mut iwram = Box::new([0u8; 0x8000]);
        iwram[0x7FFC] = 0x3C;
        iwram[0x7FFD] = 0x01;
        iwram[0x7FFE] = 0x00;
        iwram[0x7FFF] = 0x00;

        let mut io = Box::new([0u8; 0x400]);
        io[0] = 0x80;

        Self {
            bios,
            bios_read_return: 0xE129F000,
            use_real_bios: false,
            intrwait_flag_addr: 0,
            intrwait_active: false,
            wram: Box::new([0u8; 0x40000]),
            iwram,
            io: Box::new([0u8; 0x400]),
            vblank_io_snapshot: None,
            palette: Box::new([0u8; 0x400]),
            vram: Box::new([0u8; 0x18000]),
            oam: Box::new([0u8; 0x400]),
            sram: Box::new([0xFFu8; 0x8000]),
            rom: Vec::new(),
            waitcnt: 0x0000,
            interrupt: InterruptController::new(),
            halt_pending: false,
            vram_dirty: true,
            oam_dirty: true,
            palette_dirty: true,
            io_ppu_dirty: true,
            io_timer_dirty: true,
            io_dma_dirty: true,
            dma_active: false,
            save_type: SaveType::None,
            flash: None,
            eeprom: None,
            ewram_write_limit: None,
            vram_write_log: Vec::new(),
            vram_log_enabled: false,
            ewram_tile_log: Vec::new(),
            ewram_tile_log_enabled: false,
            palette_write_log: Vec::new(),
            palette_log_enabled: false,
            dma_log: Vec::new(),
            dma_log_enabled: false,
            vram_log_pc: 0,
            swi_log: Vec::new(),
            swi_log_enabled: false,
            arm_swi_count: 0,
            thumb_swi_count: 0,
            cpu_set_log: Vec::new(),
            cpu_set_log_enabled: false,
            pc_trace_base: 0,
            pc_trace_counts: Vec::new(),
            irq_trace: Vec::new(),
            irq_trace_enabled: false,
            iwram_write_log: Vec::new(),
            iwram_write_log_enabled: false,
            ewram_range_log: Vec::new(),
            ewram_range_log_enabled: false,
            reg_snapshots: Vec::new(),
            reg_snapshot_enabled: false,
            decomp_writes: Vec::new(),
            decomp_writes_enabled: false,
            keyinput_read_trace_enabled: false,
            keyinput_read_pcs: Vec::new(),
            keyinput_read_trace_pc: 0,
            timer_writes: Vec::new(),
            timer_writes_enabled: false,
            input_reads: Vec::new(),
            input_reads_enabled: false,
            dispcnt_write_log: Vec::new(),
            dispcnt_write_log_enabled: false,
            ie_ime_write_log: Vec::new(),
            ie_ime_write_log_enabled: false,
        }
    }

    pub fn reset(&mut self) {
        self.wram.fill(0);
        self.iwram.fill(0);
        self.io.fill(0);
        self.palette.fill(0);
        self.vram.fill(0);
        self.oam.fill(0);
        self.sram.fill(0);
        self.waitcnt = 0x0000;
        self.interrupt.reset();
        if let Some(ref mut flash) = self.flash {
            flash.reset();
        }
        if let Some(ref mut eeprom) = self.eeprom {
            eeprom.reset();
        }
    }

    /// Clear EWRAM (0x02000000-0x0203FFFF)
    pub fn clear_ewram(&mut self) {
        self.wram.fill(0);
    }

    /// Clear IWRAM (0x03000000-0x03007FFF, except top 8 bytes)
    pub fn clear_iwram(&mut self) {
        // Don't clear the top 8 bytes (0x03007FF8-0x03007FFF) - used for BIOS communication
        self.iwram[..0x7FF8].fill(0);
    }

    /// Clear Palette (0x05000000-0x050003FF)
    pub fn clear_palette(&mut self) {
        self.palette.fill(0);
    }

    /// Clear VRAM (0x06000000-0x06017FFF)
    pub fn clear_vram(&mut self) {
        self.vram.fill(0);
    }

    /// Clear OAM (0x07000000-0x070003FF)
    pub fn clear_oam(&mut self) {
        self.oam.fill(0);
    }

    /// Clear IO registers (0x04000000-0x040003FE, except some)
    pub fn clear_io(&mut self) {
        // Don't clear some registers that should persist
        // (e.g., POSTFLG, WAITCNT, etc.)
        self.io.fill(0);
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.rom = data;
    }

    /// Set the cartridge save type
    pub fn set_save_type(&mut self, save_type: SaveType) {
        self.save_type = save_type;
        self.flash = None;
        self.eeprom = None;
        match save_type {
            SaveType::Flash64K => self.flash = Some(Flash::new_64k()),
            SaveType::Flash128K => self.flash = Some(Flash::new_128k()),
            SaveType::Eeprom512B => self.eeprom = Some(Eeprom::new_512b()),
            SaveType::Eeprom8K => self.eeprom = Some(Eeprom::new_8k()),
            _ => {}
        }
    }

    /// Get the current save type
    pub fn save_type(&self) -> SaveType {
        self.save_type
    }

    /// Load SRAM data from a save file
    pub fn load_sram(&mut self, data: &[u8]) {
        let len = data.len().min(self.sram.len());
        self.sram[..len].copy_from_slice(&data[..len]);
    }

    pub fn zero_sram(&mut self) {
        self.sram.fill(0);
        if let Some(ref mut flash) = self.flash {
            flash.data_mut().fill(0);
        }
    }

    /// Check if address is in EEPROM access range
    fn is_eeprom_access(&self, addr: u32) -> bool {
        matches!(self.save_type, SaveType::Eeprom512B | SaveType::Eeprom8K) && addr >= 0x0DFFFF00
    }

    /// Load BIOS from a file
    pub fn load_bios(&mut self, data: Vec<u8>) {
        let mut bios_data = vec![0u8; 0x4000];
        let len = data.len().min(0x4000);
        bios_data[..len].copy_from_slice(&data[..len]);
        self.bios = bios_data;
    }

    /// Check if BIOS is loaded (not all zeros)
    pub fn has_bios(&self) -> bool {
        self.bios.iter().any(|&b| b != 0)
    }

    pub fn set_bios_read_return(&mut self, val: u32) {
        self.bios_read_return = val;
    }

    pub fn get_irq_handler(&self) -> u32 {
        let off = 0x7FFC;
        u32::from_le_bytes([
            self.iwram[off],
            self.iwram[off + 1],
            self.iwram[off + 2],
            self.iwram[off + 3],
        ])
    }

    pub fn get_bios_read_return(&self) -> u32 {
        self.bios_read_return
    }

    pub fn bios_read_word(&self, offset: usize) -> u32 {
        if offset + 4 <= self.bios.len() {
            u32::from_le_bytes([
                self.bios[offset],
                self.bios[offset + 1],
                self.bios[offset + 2],
                self.bios[offset + 3],
            ])
        } else {
            0
        }
    }

    pub fn reinstall_bios_returns(&mut self) {
        let bios_return: [u8; 4] = [0x1E, 0xFF, 0x2F, 0xE1]; // BX LR
        for offset in [
            0x0C4, 0x0DC, 0x128, 0x130, 0x138, 0x140, 0x148, 0x150, 0x158, 0x164, 0x17C, 0x188,
            0x190, 0x198, 0x1A0, 0x1A8, 0x1B0, 0x1B8, 0x1C0, 0x1C8, 0x1D0, 0x1D8, 0x1E0, 0x1E8,
            0x1F0, 0x1F8, 0x200, 0x208, 0x210, 0x218, 0x220, 0x228, 0x230, 0x238, 0x240, 0x248,
            0x250, 0x258, 0x260, 0x268, 0x270, 0x278, 0x280, 0x288, 0x290, 0x298, 0x2A0, 0x2A8,
            0x2B0, 0x2B8, 0x2C0, 0x2C8, 0x2D0, 0x2D8, 0x2E0, 0x2E8, 0x2F0, 0x2F8, 0x300, 0x400,
            0x500, 0x600, 0x700, 0x800, 0x900, 0xA00, 0xB00, 0xC00, 0xD00, 0xE00, 0xF00, 0x1000,
            0x1100, 0x1200, 0x1300, 0x1400, 0x1500, 0x1600, 0x1700, 0x1800, 0x1900, 0x1A00, 0x1B00,
            0x1C00, 0x1D00, 0x1E00, 0x1F00,
            // 0x2000 is special - uses VBlank wait stub below
            0x2100, 0x2200, 0x2300, 0x2400, 0x2500, 0x2600, 0x2700, 0x2800, 0x2900, 0x2A00, 0x2B00,
            0x2C00, 0x2D00, 0x2E00, 0x2F00, 0x3000, 0x3100, 0x3200, 0x3300, 0x3400, 0x3500, 0x3600,
            0x3700, 0x3800, 0x3900, 0x3A00, 0x3B00, 0x3C00, 0x3D00, 0x3E00, 0x3F00,
        ]
        .iter()
        {
            if *offset + 4 <= 0x4000 {
                self.bios[*offset..(*offset + 4)].copy_from_slice(&bios_return);
            }
        }

        // BIOS function at 0x2000: simple BX LR return stub
        // Games call this via BX directly (not SWI) - must not corrupt stack
        // Previous STMFD/LDMFD stub was broken: BX LR returns before LDMFD pops,
        // leaving SP decremented by 4 bytes each call, corrupting caller's stack.
        if 0x2004 <= 0x4000 {
            self.bios[0x2000..0x2004].copy_from_slice(&bios_return);
        }

        // BIOS IRQ return stub at 0x3000:
        // When take_interrupt fires, it pushes the return address onto the
        // IRQ stack and sets LR=0x3000. The user ISR returns with BX LR,
        // landing here. This stub pops the saved return address and does
        // a proper exception return (restores CPSR from SPSR_irq).
        //   0x3000: LDMFD SP!, {LR}     ; pop saved return address
        //   0x3004: SUBS PC, LR, #4     ; return from exception
        let irq_return_stub: [u8; 8] = [
            0x00, 0x40, 0xBD, 0xE8, // LDMIA SP!, {LR} = 0xE8BD4000 (LDMFD = pop)
            0x04, 0xF0, 0x5E, 0xE2, // SUBS PC, LR, #4 = 0xE25EF004
        ];
        if 0x3008 <= 0x4000 {
            self.bios[0x3000..0x3008].copy_from_slice(&irq_return_stub);
        }

        // BIOS IRQ dispatch is installed in new() at 0x0018-0x004B
        // Do not overwrite it here
    }

    /// Get access cycles for a memory region
    pub fn get_access_cycles(&self, addr: u32, _sequential: bool) -> u32 {
        match addr {
            0x0000_0000..=0x0000_3FFF => 2, // BIOS: always 2 cycles
            0x0200_0000..=0x0203_FFFF => 3, // WRAM: always 3 cycles
            0x0300_0000..=0x0300_7FFF => 1, // IWRAM: always 1 cycle
            0x0400_0000..=0x0400_03FE => 1, // IO: always 1 cycle
            0x0500_0000..=0x0500_03FF => 1, // Palette: always 1 cycle
            0x0600_0000..=0x0601_7FFF => 1, // VRAM: always 1 cycle
            0x0700_0000..=0x0700_03FF => 1, // OAM: always 1 cycle
            0x0800_0000..=0x09FF_FFFF => self.get_rom_waitstates(0, _sequential),
            0x0A00_0000..=0x0BFF_FFFF => self.get_rom_waitstates(1, _sequential),
            0x0C00_0000..=0x0DFF_FFFF => self.get_rom_waitstates(2, _sequential),
            _ => 1, // Unknown region
        }
    }

    fn get_rom_waitstates(&self, region: usize, sequential: bool) -> u32 {
        // Extract waitstate settings from WAITCNT register
        let ws = if sequential {
            match region {
                0 => (self.waitcnt >> 2) & 0x3,
                1 => (self.waitcnt >> 6) & 0x3,
                2 => (self.waitcnt >> 10) & 0x3,
                _ => 3,
            }
        } else {
            match region {
                0 => self.waitcnt & 0x3,
                1 => (self.waitcnt >> 4) & 0x3,
                2 => (self.waitcnt >> 8) & 0x3,
                _ => 3,
            }
        };

        // Convert waitstate to cycles (WS0=3, WS1=2, WS2=1, WS3=1? - varies)
        // Simplified: default to 3 cycles
        match ws {
            0 => 3,
            1 => 2,
            2 => 1,
            _ => 3,
        }
    }

    /// Map address to actual memory location
    fn map_address(&self, addr: u32) -> (MemoryRegion, usize) {
        match addr {
            0x0000_0000..=0x0000_3FFF => (MemoryRegion::Bios, (addr - 0x0000_0000) as usize),
            // EWRAM (256KB) and its mirrors
            0x0200_0000..=0x02FF_FFFF => {
                let offset = ((addr - 0x0200_0000) & 0x3_FFFF) as usize; // Mask to 256KB
                (MemoryRegion::Wram, offset)
            }
            // IWRAM (32KB) and its mirrors
            0x0300_0000..=0x03FF_FFFF => {
                let offset = ((addr - 0x0300_0000) & 0x7FFF) as usize; // Mask to 32KB
                (MemoryRegion::Iwram, offset)
            }
            0x0400_0000..=0x0400_03FE => (MemoryRegion::Io, (addr - 0x0400_0000) as usize),
            // Palette (1KB) and its mirrors (every 0x400 bytes)
            0x0500_0000..=0x050F_FFFF => {
                (MemoryRegion::Palette, ((addr - 0x0500_0000) % 0x400) as usize)
            }
            // VRAM (96KB) and its mirrors (128KB period)
            // 0x00000-0x0FFFF: BG VRAM (64KB)
            // 0x10000-0x17FFF: OBJ VRAM (32KB)
            // 0x18000-0x1FFFF: mirrors 0x10000-0x17FFF (OBJ)
            // Pattern repeats every 128KB (0x20000)
            0x0600_0000..=0x060F_FFFF => {
                let raw = (addr - 0x0600_0000) % 0x2_0000;
                let offset = if raw >= 0x1_8000 { raw - 0x8000 } else { raw };
                (MemoryRegion::Vram, offset as usize)
            }
            // OAM (1KB) and its mirrors
            0x0700_0000..=0x0700_03FF => (MemoryRegion::Oam, (addr - 0x0700_0000) as usize),
            0x0700_0400..=0x070F_FFFF => {
                let offset = ((addr - 0x0700_0000) & 0x3FF) as usize; // Mask to 1KB
                (MemoryRegion::Oam, offset)
            }
            // SRAM (32KB) and its mirrors
            0x0E00_0000..=0x0FFF_FFFF => {
                let offset = ((addr - 0x0E00_0000) & 0xFFFF) as usize;
                (MemoryRegion::Sram, offset)
            }
            // ROM WS0 (0x08000000-0x09FFFFFF)
            0x0800_0000..=0x09FF_FFFF => {
                let offset = (addr - 0x0800_0000) as usize;
                (MemoryRegion::Rom, offset)
            }
            // ROM WS1 (0x0A000000-0x0BFFFFFF)
            0x0A00_0000..=0x0BFF_FFFF => {
                let offset = (addr - 0x0A00_0000) as usize;
                (MemoryRegion::Rom, offset)
            }
            // ROM WS2 (0x0C000000-0x0DFFFFFF)
            0x0C00_0000..=0x0DFF_FFFF => {
                let offset = (addr - 0x0C00_0000) as usize;
                (MemoryRegion::Rom, offset)
            }
            _ => (MemoryRegion::Unknown, 0),
        }
    }

    /// Read a byte from memory
    pub fn read_byte(&mut self, addr: u32) -> u8 {
        let (region, offset) = self.map_address(addr);

        match region {
            MemoryRegion::Bios => self.bios[offset],
            MemoryRegion::Wram => {
                if self.input_reads_enabled && self.input_reads.len() < 100_000 {
                    if addr >= 0x02008CF8 && addr < 0x02008D10 {
                        self.input_reads.push((addr, self.vram_log_pc));
                    }
                }
                self.wram[offset]
            }
            MemoryRegion::Iwram => self.iwram[offset],
            MemoryRegion::Io => self.read_io(addr),
            MemoryRegion::Palette => self.palette[offset],
            MemoryRegion::Vram => self.vram[offset],
            MemoryRegion::Oam => self.oam[offset],
            MemoryRegion::Sram => match self.save_type {
                SaveType::Sram | SaveType::None => self.sram[offset & 0x7FFF],
                SaveType::Flash64K | SaveType::Flash128K => {
                    self.flash.as_ref().map_or(0xFF, |f| f.read(offset as u32))
                }
                _ => 0xFF,
            },
            MemoryRegion::Rom => {
                if self.is_eeprom_access(addr) {
                    return self.eeprom.as_mut().map_or(0xFF, |e| e.serial_read());
                }
                if self.rom.is_empty() {
                    0
                } else if offset < self.rom.len() {
                    self.rom[offset]
                } else {
                    ((addr >> 1) >> (8 * (addr & 1))) as u8
                }
            }
            MemoryRegion::Unknown => 0,
        }
    }

    /// Write a byte to memory (internal, used by write_word)
    fn write_byte_internal(&mut self, addr: u32, val: u8) {
        let (region, offset) = self.map_address(addr);

        match region {
            MemoryRegion::Bios => {
                // BIOS is read-only
            }
            MemoryRegion::Wram => {
                if let Some(limit) = self.ewram_write_limit {
                    if addr >= limit {
                        return;
                    }
                }
                if self.ewram_range_log_enabled && self.ewram_range_log.len() < 500_000 {
                    if addr >= 0x0200_0000 && addr < 0x0201_0000 {
                        self.ewram_range_log.push((addr, self.vram_log_pc, val));
                    }
                }
                if addr >= 0x02000050 && addr <= 0x02000053 {
                    self.keyinput_read_pcs
                        .push(0x01000000 | (self.vram_log_pc & 0x0FFFFFFF));
                }
                if self.timer_writes_enabled && self.timer_writes.len() < 100_000 {
                    if addr >= 0x02000050 && addr <= 0x02000053 {
                        self.timer_writes.push((addr, self.vram_log_pc, val));
                    }
                    if addr >= 0x02000074 && addr <= 0x02000077 {
                        self.timer_writes
                            .push((addr | 0x02000000, self.vram_log_pc, val));
                    }
                }
                if self.ewram_tile_log_enabled && self.ewram_tile_log.len() < 500_000 {
                    if addr >= 0x0200_0000 && addr < 0x0204_0000 && val != 0 {
                        self.ewram_tile_log.push((addr, self.vram_log_pc, val));
                    }
                }
                if self.decomp_writes_enabled && self.decomp_writes.len() < 500_000 {
                    let pc = self.vram_log_pc;
                    if pc >= 0x080D0900 && pc < 0x080D0C20 {
                        self.decomp_writes.push((addr, pc, val));
                    }
                }
                self.wram[offset] = val
            }
            MemoryRegion::Iwram => {
                if self.iwram_write_log_enabled && self.iwram_write_log.len() < 500_000 {
                    if addr >= 0x03006DD8 && addr < 0x03006F59 {
                        self.iwram_write_log.push((addr, self.vram_log_pc, val));
                    }
                }
                if offset >= 0x7FF8 && offset <= 0x7FFB && self.iwram_write_log.len() < 500_000 {
                    self.iwram_write_log.push((addr, self.vram_log_pc, val));
                }
                self.iwram[offset] = val;
            }
            MemoryRegion::Io => self.write_io(addr, val),
            MemoryRegion::Palette => {
                if self.palette_log_enabled && self.palette_write_log.len() < 1_000_000 {
                    self.palette_write_log.push((
                        addr,
                        self.vram_log_pc << 1,
                        val,
                        self.dma_active,
                    ));
                }
                self.palette[offset] = val
            }
            MemoryRegion::Vram => {
                if self.vram_log_enabled && self.vram_write_log.len() < 100_000 {
                    self.vram_write_log.push((addr, self.vram_log_pc, val));
                }
                self.vram[offset] = val
            }
            MemoryRegion::Oam => self.oam[offset] = val,
            MemoryRegion::Sram => match self.save_type {
                SaveType::Sram | SaveType::None => self.sram[offset & 0x7FFF] = val,
                SaveType::Flash64K | SaveType::Flash128K => {
                    self.flash.as_mut().map(|f| f.write(offset as u32, val));
                }
                _ => {}
            },
            MemoryRegion::Rom => {
                // EEPROM write interception
                if self.is_eeprom_access(addr) {
                    self.eeprom.as_mut().map(|e| e.serial_write(val));
                }
                // ROM is otherwise read-only
            }
            MemoryRegion::Unknown => {}
        }

        // Set dirty flags based on written address
        match addr {
            0x06000000..=0x06017FFF => self.vram_dirty = true,
            0x07000000..=0x070003FF => self.oam_dirty = true,
            0x05000000..=0x050003FF => self.palette_dirty = true,
            0x04000000..=0x04000055 => self.io_ppu_dirty = true,
            0x04000100..=0x0400010F => self.io_timer_dirty = true,
            0x040000B0..=0x040000DF => self.io_dma_dirty = true,
            _ => {}
        }
    }

    /// Write a byte to memory (public, handles OAM and VRAM byte-write restrictions)
    pub fn write_byte(&mut self, addr: u32, val: u8) {
        let (region, offset) = self.map_address(addr);

        // OAM ignores byte writes (only accepts 16-bit or 32-bit aligned writes)
        if region == MemoryRegion::Oam {
            return;
        }

        // VRAM: byte writes are expanded to halfwords (duplicated in both bytes)
        // This happens in ALL modes according to GBA behavior
        if region == MemoryRegion::Vram {
            let half_offset = offset & !1; // Align to halfword boundary
            let half_val = ((val as u16) << 8) | (val as u16); // Duplicate byte
            self.vram[half_offset] = (half_val & 0xFF) as u8;
            self.vram[half_offset + 1] = ((half_val >> 8) & 0xFF) as u8;
            return;
        }

        // Palette RAM: byte writes are expanded to halfwords (duplicated in both bytes)
        if region == MemoryRegion::Palette {
            let half_offset = offset & !1; // Align to halfword boundary
            let half_val = ((val as u16) << 8) | (val as u16); // Duplicate byte
            self.palette[half_offset] = (half_val & 0xFF) as u8;
            self.palette[half_offset + 1] = ((half_val >> 8) & 0xFF) as u8;
            return;
        }

        self.write_byte_internal(addr, val);
    }

    /// Read a halfword (16-bit) from memory
    pub fn read_half(&mut self, addr: u32) -> u16 {
        if addr >= 0x0E00_0000 && addr < 0x1000_0000 {
            let b = self.read_byte(addr);
            return u16::from_le_bytes([b, b]);
        }
        let aligned = addr & !1;
        let low = self.read_byte(aligned);
        let high = self.read_byte(aligned.wrapping_add(1));
        u16::from_le_bytes([low, high])
    }

    pub fn read_half_rotated(&mut self, addr: u32) -> u32 {
        if addr >= 0x0E00_0000 && addr < 0x1000_0000 {
            let b = self.read_byte(addr) as u32;
            return b | (b << 8);
        }
        let aligned = addr & !1;
        let low = self.read_byte(aligned) as u32;
        let high = self.read_byte(aligned.wrapping_add(1)) as u32;
        let val = low | (high << 8);
        let rotate = ((addr & 1) * 8) as u32;
        val.rotate_right(rotate)
    }

    /// Write a halfword (16-bit) to memory
    pub fn write_half(&mut self, addr: u32, val: u16) {
        if addr >= 0x0E00_0000 && addr < 0x1000_0000 {
            let byte_index = (addr & 1) as usize;
            let byte_val = val.to_le_bytes()[byte_index];
            self.write_byte_internal(addr, byte_val);
            return;
        }
        let addr = addr & !1;
        let bytes = val.to_le_bytes();
        self.write_byte_internal(addr, bytes[0]);
        self.write_byte_internal(addr + 1, bytes[1]);
    }

    /// Read a word from memory (optimized fast path for ROM/IWRAM)
    #[inline(always)]
    pub fn read_word_fast(&mut self, addr: u32) -> u32 {
        match addr {
            // ROM WS0 - most common for instruction fetch
            0x0800_0000..=0x09FF_FFFF => {
                let offset = (addr - 0x0800_0000) as usize;
                if offset + 3 < self.rom.len() {
                    unsafe {
                        let ptr = self.rom.as_ptr().add(offset);
                        u32::from_le_bytes([*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)])
                    }
                } else {
                    self.read_word(addr)
                }
            }
            // IWRAM - fast access for stack
            0x0300_0000..=0x03FF_FFFF => {
                let offset = (addr - 0x0300_0000) as usize;
                if offset + 3 < self.iwram.len() {
                    unsafe {
                        let ptr = self.iwram.as_ptr().add(offset);
                        u32::from_le_bytes([*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)])
                    }
                } else {
                    self.read_word(addr)
                }
            }
            // WRAM
            0x0200_0000..=0x02FF_FFFF => {
                let offset = (addr - 0x0200_0000) as usize;
                if offset + 3 < self.wram.len() {
                    unsafe {
                        let ptr = self.wram.as_ptr().add(offset);
                        u32::from_le_bytes([*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)])
                    }
                } else {
                    self.read_word(addr)
                }
            }
            _ => self.read_word(addr),
        }
    }

    /// Read a word (32-bit) from memory
    pub fn read_word(&mut self, addr: u32) -> u32 {
        if addr >= 0x0E00_0000 && addr < 0x1000_0000 {
            let b = self.read_byte(addr) as u32;
            return b | (b << 8) | (b << 16) | (b << 24);
        }
        if addr & 3 != 0 {
            // Unaligned read - rotate
            let aligned = addr & !3;
            let low = self.read_half(aligned) as u32;
            let high = self.read_half(aligned.wrapping_add(2)) as u32;
            let val = low | (high << 16);
            val.rotate_right(8 * (addr & 3) as u32)
        } else {
            let b0 = self.read_byte(addr) as u32;
            let b1 = self.read_byte(addr.wrapping_add(1)) as u32;
            let b2 = self.read_byte(addr.wrapping_add(2)) as u32;
            let b3 = self.read_byte(addr.wrapping_add(3)) as u32;
            b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
        }
    }

    /// Write a word (32-bit) to memory
    pub fn write_word(&mut self, addr: u32, val: u32) {
        if addr >= 0x0E00_0000 && addr < 0x1000_0000 {
            let byte_index = (addr & 3) as usize;
            let byte_val = val.to_le_bytes()[byte_index];
            self.write_byte_internal(addr, byte_val);
            return;
        }
        let addr = addr & !3;
        let bytes = val.to_le_bytes();
        for i in 0..4usize {
            self.write_byte_internal(addr.wrapping_add(i as u32), bytes[i]);
        }
    }

    /// Read from IO register
    fn read_io(&mut self, addr: u32) -> u8 {
        let offset = (addr - 0x0400_0000) as usize;

        // Handle interrupt registers (IE is at 0x0400_0200, not 0x0400_0000!)
        let (int_offset, byte_index) = match addr {
            0x0400_0200 | 0x0400_0201 => (Some(0x200), (addr & 1) as usize), // IE
            0x0400_0202 | 0x0400_0203 => (Some(0x002), (addr & 1) as usize), // IF
            0x0400_0208 => (Some(0x208), 0),                                 // IME
            _ => (None, 0),
        };

        if let Some(ioff) = int_offset {
            let val = self.interrupt.read_register(ioff);
            return if ioff == 0x208 {
                val as u8
            } else {
                (val >> (8 * byte_index as u32)) as u8
            };
        }

        match offset {
            0x000 => self.io[offset], // DISPCNT
            0x004 => self.io[offset], // DISPSTAT
            0x006 => self.io[offset], // VCOUNT (would be updated by PPU)
            0x130 | 0x131 => {
                let v = self.io[offset];
                if self.keyinput_read_trace_enabled && self.keyinput_read_pcs.len() < 1000 {
                    self.keyinput_read_pcs.push(self.vram_log_pc);
                }
                v
            }
            0x132 | 0x133 => self.io[offset], // KEYCNT
            _ => self.io[offset],
        }
    }

    /// Write to IO register
    fn write_io(&mut self, addr: u32, val: u8) {
        let offset = (addr - 0x0400_0000) as usize;

        // Handle interrupt registers (IE is at 0x0400_0200, not 0x0400_0000!)
        let (int_offset, byte_index) = match addr {
            0x0400_0200 | 0x0400_0201 => (Some(0x200), (addr & 1) as usize), // IE
            0x0400_0202 | 0x0400_0203 => (Some(0x002), (addr & 1) as usize), // IF
            0x0400_0208 => (Some(0x208), 0),                                 // IME
            _ => (None, 0),
        };

        if let Some(ioff) = int_offset {
            let current = self.interrupt.read_register(ioff);
            let new_val = if ioff == 0x208 {
                val as u16
            } else {
                let shift = 8 * byte_index as u32;
                let mask = 0xFF << shift;
                let current_cleared = current & !mask;
                current_cleared | ((val as u16) << shift)
            };
            if self.ie_ime_write_log_enabled && self.ie_ime_write_log.len() < 10_000 {
                self.ie_ime_write_log
                    .push((self.vram_log_pc, addr, new_val));
            }
            self.interrupt.write_register(ioff, new_val);
            return;
        }

        match offset {
            0x204 => {
                // WAITCNT - only some bits are writable
                self.waitcnt = u16::from_le_bytes([val, self.io[offset + 1]]);
            }
            0x301 => {
                // HALTCNT - halt the CPU
                // Writing 0 to bit 0 enters HALT mode
                self.halt_pending = true;
                self.io[offset] = val;
            }
            0x000..=0x003 => {
                if self.dispcnt_write_log_enabled && self.dispcnt_write_log.len() < 10_000 {
                    self.dispcnt_write_log
                        .push((self.vram_log_pc, offset as u8, val));
                }
                self.io[offset] = val;
            }
            _ => {
                self.io[offset] = val;
            }
        }
    }

    /// Read a palette color entry (16-bit RGB555)
    /// pal_num: 0 for BG palette, 1 for OBJ palette
    /// index: color index (0-255)
    pub fn read_palette_color(&self, pal_num: usize, index: u16) -> u16 {
        // Palette RAM is at 0x0500_0000
        // BG palette: 0x0500_0000 - 0x0500_01FF (512 bytes, 256 colors)
        // OBJ palette: 0x0500_0200 - 0x0500_03FF (512 bytes, 256 colors)
        let offset = if pal_num == 0 {
            // BG palette
            (index as usize * 2) & 0x3FF
        } else {
            // OBJ palette (offset by 0x200)
            0x200 + ((index as usize * 2) & 0x1FF)
        };

        if offset + 1 < self.palette.len() {
            u16::from_le_bytes([self.palette[offset], self.palette[offset + 1]])
        } else {
            0
        }
    }

    /// Get a reference to the palette RAM
    pub fn palette(&self) -> &[u8; 0x400] {
        &self.palette
    }

    /// Get a mutable reference to BIOS data (for font embedding)
    pub fn bios_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bios
    }

    /// Get a reference to EWRAM data
    pub fn wram(&self) -> &[u8] {
        &self.wram[..]
    }

    /// Get a reference to VRAM data
    pub fn vram(&self) -> &[u8] {
        &self.vram[..]
    }

    pub fn rom(&self) -> &[u8] {
        &self.rom[..]
    }

    pub fn iwram(&self) -> &[u8] {
        &self.iwram[..]
    }

    pub fn iwram_mut(&mut self) -> &mut [u8] {
        &mut self.iwram[..]
    }

    /// Get a reference to OAM data
    pub fn oam(&self) -> &[u8] {
        &self.oam[..]
    }

    /// Get a reference to IO register data
    pub fn io(&self) -> &[u8] {
        &self.io[..]
    }

    /// Get a mutable reference to IO register data
    pub fn io_mut(&mut self) -> &mut [u8] {
        &mut self.io[..]
    }

    /// Check if address is in interrupt register range (0x0400_0000 - 0x0400_0208)
    pub fn is_interrupt_register(addr: u32) -> bool {
        matches!(addr, 0x0400_0000..=0x0400_0208)
    }

    /// Get interrupt register offset (returns 0xFFFF if not in range)
    pub fn get_interrupt_register_offset(addr: u32) -> Option<usize> {
        match addr {
            0x0400_0000 | 0x0400_0001 => Some(0x000), // IE
            0x0400_0002 | 0x0400_0003 => Some(0x002), // IF
            0x0400_0208 => Some(0x208),               // IME
            _ => None,
        }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryRegion {
    Bios,
    Wram,
    Iwram,
    Io,
    Palette,
    Vram,
    Oam,
    Sram,
    Rom,
    Unknown,
}
