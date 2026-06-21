//! GBA EEPROM Save Memory
//!
//! EEPROM is accessed via a serial protocol through the highest ROM mirror address.
//! Supports 512B (14-bit address) and 8KB (6-bit address) EEPROM sizes.

/// EEPROM serial interface
pub struct Eeprom {
    data: Vec<u8>,
    #[allow(dead_code)]
    size: usize,
    address_bits: usize, // 14 for 512B, 6 for 8KB

    // Serial state machine
    state: EepromState,
    shift_reg: u64,
    bits_received: usize,
    write_buffer: Vec<u8>,
    write_offset: usize,
    read_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EepromState {
    Idle,
    Command,   // Receiving command bits (2 bits: read=10, write=01)
    Address,   // Receiving address bits
    WriteData, // Receiving write data
    ReadData,  // Sending read data
}

impl Eeprom {
    pub fn new_512b() -> Self {
        Self {
            data: vec![0xFF; 512],
            size: 512,
            address_bits: 14,
            state: EepromState::Idle,
            shift_reg: 0,
            bits_received: 0,
            write_buffer: Vec::new(),
            write_offset: 0,
            read_offset: 0,
        }
    }

    pub fn new_8k() -> Self {
        Self {
            data: vec![0xFF; 8192],
            size: 8192,
            address_bits: 6,
            state: EepromState::Idle,
            shift_reg: 0,
            bits_received: 0,
            write_buffer: Vec::new(),
            write_offset: 0,
            read_offset: 0,
        }
    }

    pub fn reset(&mut self) {
        self.state = EepromState::Idle;
        self.shift_reg = 0;
        self.bits_received = 0;
        self.write_buffer.clear();
    }

    /// Read one bit from EEPROM (for DMA reads)
    /// Returns the next bit of the read data
    pub fn serial_read(&mut self) -> u8 {
        match self.state {
            EepromState::ReadData => {
                if self.read_offset < self.data.len() {
                    // Read bits MSB first from current byte
                    let byte = self.data[self.read_offset];
                    let bit = (byte >> 7) & 1;
                    // Shift the byte left
                    self.data[self.read_offset] = byte << 1;
                    self.bits_received += 1;
                    // After 8 bits, advance to next byte
                    if self.bits_received >= 8 {
                        self.bits_received = 0;
                        self.read_offset += 1;
                        // After reading all data, return to idle
                        if self.read_offset >= self.data.len() {
                            self.state = EepromState::Idle;
                        }
                    }
                    bit
                } else {
                    self.state = EepromState::Idle;
                    1
                }
            }
            _ => 1, // Default: high (idle bus)
        }
    }

    /// Write one bit to EEPROM (from DMA writes or memory writes)
    pub fn serial_write(&mut self, bit: u8) {
        let bit = bit & 1;

        match self.state {
            EepromState::Idle => {
                // Wait for start bit (1)
                if bit == 1 {
                    self.shift_reg = 0;
                    self.bits_received = 0;
                    self.state = EepromState::Command;
                }
            }
            EepromState::Command => {
                // Receive 2-bit command: 10=read, 01=write
                self.shift_reg = (self.shift_reg << 1) | bit as u64;
                self.bits_received += 1;
                if self.bits_received >= 2 {
                    let cmd = (self.shift_reg & 0x3) as u8;
                    self.shift_reg = 0;
                    self.bits_received = 0;
                    match cmd {
                        0b10 => {
                            self.state = EepromState::Address;
                        }
                        0b01 => {
                            self.write_buffer.clear();
                            self.state = EepromState::Address;
                        }
                        _ => {
                            self.state = EepromState::Idle;
                        }
                    }
                }
            }
            EepromState::Address => {
                // Receive address bits
                self.shift_reg = (self.shift_reg << 1) | bit as u64;
                self.bits_received += 1;
                if self.bits_received >= self.address_bits {
                    let addr = (self.shift_reg as usize) % self.data.len();
                    let cmd = (self.shift_reg >> self.address_bits) as u8;
                    self.shift_reg = 0;
                    self.bits_received = 0;

                    if cmd == 0b10 {
                        // Read command
                        self.read_offset = addr;
                        self.state = EepromState::ReadData;
                    } else {
                        // Write command
                        self.write_offset = addr;
                        self.state = EepromState::WriteData;
                    }
                }
            }
            EepromState::WriteData => {
                // Receive 8-byte blocks
                self.shift_reg = (self.shift_reg << 1) | bit as u64;
                self.bits_received += 1;
                if self.bits_received >= 8 {
                    let byte = (self.shift_reg & 0xFF) as u8;
                    self.write_buffer.push(byte);
                    self.shift_reg = 0;
                    self.bits_received = 0;

                    // Write 8 bytes at a time
                    if self.write_buffer.len() >= 8 {
                        for (i, &byte) in self.write_buffer.iter().enumerate() {
                            let addr = (self.write_offset + i) % self.data.len();
                            self.data[addr] = byte;
                        }
                        self.write_offset += 8;
                        self.write_buffer.clear();
                        self.state = EepromState::Idle;
                    }
                }
            }
            EepromState::ReadData => {
                // Read is handled by serial_read
            }
        }
    }

    /// Load data from a save file
    pub fn load(&mut self, data: &[u8]) {
        let len = data.len().min(self.data.len());
        self.data[..len].copy_from_slice(&data[..len]);
    }

    /// Get a reference to the raw data (for saving)
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
