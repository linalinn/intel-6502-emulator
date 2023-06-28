use std::ops::{Index, IndexMut};
use std::str::Bytes;


type BYTE = u8;
type WORD = u16;

#[derive(Debug)]
struct MEM {
    data: [BYTE; 1024 * 64]
}

impl MEM {
    fn write_word(&mut self, data: WORD, addr: u32, cycles: &mut u32) {
        self[addr as usize] = (data & 0xFF) as u8;
        self[addr as usize + 1] = (data >> 8) as u8;
        *cycles -= 2;
    }
}

impl Index<usize> for MEM {
    type Output = BYTE;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for MEM {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

#[derive(Debug)]
struct CPU {
    // program counter
    pc: WORD,
    // stack pointer
    sp: WORD,
    // registers
    a: BYTE, 
    x: BYTE,
    y: BYTE,
    C: BYTE, // Carry flag
    Z: bool, // Zero Flag
    I: BYTE, // IRQ Disable flag
    D: BYTE, // Decimal mode flag
    B: BYTE, // Break command flag
    V: BYTE, // overflow flag
    N: bool, // negative flag
}

impl CPU {
    // opcodes
    const INS_LDA_IM :BYTE = 0xA9;
    const INS_LDA_ZP :BYTE = 0xA5;

    fn reset(&mut self, mem: &mut MEM) {
        self.pc = 0xFFFC;
        self.sp = 0x00FF;
        self.D = 0;
        self.a = 0;
        self.x = 0;
        self.y = 0
    }

    

    fn fetch_byte(&mut self, cycles: &mut u32, mem: &mut MEM) -> BYTE {
        let data = mem[self.pc as usize];
        self.pc += 1;
        *cycles -= 1;
        return data;
    }

    fn fetch_word(&mut self, cycles: &mut u32, mem: &mut MEM) -> WORD {
        // 6502 is little endian
        let mut data = mem[self.pc as usize] as u16;
        self.pc += 1;
        *cycles -= 1;
        data += (mem[self.pc as usize] as u16) << 8;
        self.pc += 1;
        *cycles -= 1;
        return data;
    }

    fn read_byte(&mut self, cycles: &mut u32, addr: BYTE, mem: &mut MEM) -> BYTE {
        let data = mem[addr as usize];
        self.pc += 1;
        *cycles -= 1;
        return data;
    }

    fn lda_set_status(&mut self) {
        self.Z = (self.a == 0);
        self.N = (self.a & 0b10000000) > 0;
    }

    fn exec(&mut self, cycles: &mut u32, mem: &mut MEM) {
        while *cycles > 0 {
            let instruction: BYTE = self.fetch_byte(cycles, mem);
            match instruction {
                0xA9 => { // const don't work INS_LDA_IM
                    let value = self.fetch_byte(cycles, mem);
                    self.a = value;
                    self.lda_set_status()
                }
                0xA5 => { // const don't work INS_LDA_ZP
                    let zero_page_addr = self.fetch_byte(cycles, mem);
                    self.a = self.read_byte(cycles, zero_page_addr, mem);
                    self.lda_set_status()
                }
                0xB5 => { // const don't work INS_LDA_ZP_X
                    let zero_page_addr_X = self.fetch_byte(cycles, mem) + self.x;
                    *cycles -= 1; // for the zero page addr + X register calulation
                    self.a = self.read_byte(cycles, zero_page_addr_X, mem);
                    self.lda_set_status()
                }
                0x20 => { // const don't work INS_JSR
                    let subAddr = self.fetch_word(cycles, mem);
                    mem.write_word(self.pc - 1, self.sp as u32, cycles);
                    self.pc = subAddr;
                    *cycles -= 1;
                }
                _=> break
            }
        }
    }
}

fn main() {
    let mut mem = MEM{data:[0; 1024 * 64]};
    let mut cpu = CPU { pc: 0, sp: 0, a: 0, x: 0, y: 0, C: 0, Z: false, I: 0, D: 0, B: 0, V: 0, N: false };
    cpu.reset( &mut mem);
    print!("CPU: {:?}", cpu);
    // start inline program
    mem[0xFFFC] = CPU::INS_LDA_ZP;
    mem[0xFFFD] = 0x42;
    mem[0x42] = 0x10;
    // end inline program
    cpu.exec( &mut 3, &mut mem);
    print!("MEM: {:?}", mem);
    print!("CPU: {:?}", cpu);
}
