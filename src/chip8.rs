use num::{CheckedAdd, PrimInt};
use num::CheckedSub;
use rand::Rng;
use std::ops::BitAnd;
use std::fmt::Display;
use crate::chip8_util::validate_argument;

pub struct Chip8 {
    // We should break this into cohesive components
    memory: [u8; 4096],
    cpu_registers: [u8; 16],
    opcode: u8,
    index_register: u16,
    program_counter: u16,
    pub gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack_data: Vec<u16>,
    sp: u16,
    key_states: u16,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0; 4096],
            cpu_registers: [0; 16],
            opcode: 0,
            index_register: 0,
            program_counter: 0x200, // CHIP8 expects PC to start at 0x200
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack_data: vec![0; 16],
            sp: 0,
            key_states: 0,
        }
    }

    pub fn fetch_cycle(&mut self) {
        let opcode: u16 = self.fetch_next();
        self.execute_instruction(opcode);
    }

    // Executes the given opcode
    // Includes decoding and executing the given opcode
    pub fn execute_instruction(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x0E0 => self.clear_screen(),
                    0x0EE => self.subroutine_return(),
                    _ => panic!("Unknown opcode: {}", opcode)
                }
            },
            0x1000 => self.jump_to_address(opcode & 0x0FFF),
            0x2000 => self.call_address(opcode & 0x0FFF),
            0x3000 => self.skip_next_if_byte_is_vx(
                ((opcode & 0x0F00) >> 8) as u8, // XKK
                (opcode & 0x0FF) as u8),
            0x4000 => self.skip_next_if_byte_is_not_vx(
                ((opcode & 0x0F00) >> 8) as u8, // XKK
                (opcode & 0x0FF) as u8),
            0x5000 => self.skip_next_if_vx_eql_vy(
                ((opcode & 0xF00) >> 8) as u8, // XY0
                ((opcode & 0x0F0) >> 4) as u8),
            0x6000 => self.set_register_value(      // XKK
                ((opcode & 0xF00) >> 8) as u8,
                ((opcode & 0x0FF) as u8)
            ),
            0x7000 => self.add(      // XKK
                ((opcode & 0xF00) >> 8) as u8,
                ((opcode & 0x0FF) as u8)
            ),
            0x8000 => {
                match opcode & 0xF00F {
                    0x8000 => self.load(((opcode & 0x0F00) >> 8) as u8, ((opcode & 0x00F0) >> 4) as u8),
                    0x8001 => self.bit_or(((opcode & 0x0F00) >> 8) as u8, ((opcode & 0x00F0) >> 4) as u8),
                    0x8002 => self.bit_and(((opcode & 0x0F00) >> 8) as u8, ((opcode & 0x00F0) >> 4) as u8),
                    0x8003 => self.bit_xor(((opcode & 0x0F00) >> 8) as u8, ((opcode & 0x00F0) >> 4) as u8),
                    0x8004 => self.add_registers(((opcode & 0x0F00) >> 8) as u8, ((opcode & 0x00F0) >> 4) as u8),
                    0x8005 => self.sub_registers(((opcode & 0x0F00) >> 8) as u8, ((opcode & 0x00F0) >> 4) as u8),
                    0x8006 => self.shift_right_register(((opcode & 0x0F00) >> 8) as u8),
                    0x8007 => self.sub_registers_not(((opcode & 0x0F00) >> 8) as u8, ((opcode & 0x00F0) >> 4) as u8),
                    0x800E => self.shift_left_register(((opcode & 0x0F00) >> 8) as u8),
                    _ => panic!("Unknown opcode: {}", opcode)

                }
            },
            0xA000 => self.set_index_register(opcode & 0x0FFF),
            _ => {
                panic!("Unknown opcode: {}", opcode);
            }
        }
    }

    // essentially combine PC: u8 and PC+1: u8 into one u16 opcode to execute using bitshift ops
    fn fetch_next(&self) -> u16
    {
        (self.memory[self.program_counter as usize] as u16) << 8
            | self.memory[(self.program_counter + 1) as usize] as u16
    }

    //
    // Below are all the opcodes. Please refer to section 3.1
    // http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#00E0
    //

    // 00E0
    // Clears the screen.
    fn clear_screen(&mut self) {
        self.gfx = [0; 64 * 32];
    }

    // 00EE
    // Return from a subroutine
    // The interpreter sets the program counter to the address at the top of the stack,
    // then subtracts 1 from the stack pointer.
    fn subroutine_return(&mut self) {
        match self.stack_data.last() {
            Some(x) => {
                self.program_counter = *x;
                self.stack_data.pop();
            },
            None => panic!("Nothing in the stack!")
        }
    }

    // 1NNN
    // Jump Address
    // The interpreter sets the program counter to nnn
    fn jump_to_address(&mut self, address: u16) {
        self.program_counter = validate_argument(address, 0x0FFF);
    }

    // 2NNN
    // Call subroutine at nnn.
    // The interpreter increments the stack pointer,
    // then puts the current PC on the top of the stack. The PC is then set to nnn.
    fn call_address(&mut self, address: u16)
    {
        self.stack_data.push(self.program_counter);
        self.program_counter = validate_argument(address, 0x0FFF);
    }

    // 3XKK
    // Skip next instruction if VX = KK.
    // The interpreter compares register Vx to kk, and if they are equal,
    // increments the program counter by 2.
    // [todo: I incremented the program counter by 1, to be examined more]
    fn skip_next_if_byte_is_vx(&mut self, reg_x: u8, byte_value: u8)
    {
        validate_argument(reg_x, 0xF);
        validate_argument(byte_value, 0xF);
        if self.cpu_registers[reg_x as usize] == byte_value {
            self.program_counter += 1;
        }
    }

    // 4XKK
    // Skip next instruction if Vx != kk.
    // The interpreter compares register Vx to kk,
    // and if they are not equal, increments the program counter by 2.
    // todo: see todo in 3XKK
    fn skip_next_if_byte_is_not_vx(&mut self, reg_x: u8, byte_value: u8)
    {
        validate_argument(reg_x, 0xF);
        validate_argument(byte_value, 0xF);
        if self.cpu_registers[reg_x as usize] != byte_value {
            self.program_counter += 1;
        }
    }

    // 5XY0
    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal,
    // increments the program counter by 2.
    // todo: see todo in 3XKK
    fn skip_next_if_vx_eql_vy(&mut self, reg_x: u8, reg_y: u8)
    {
        validate_argument(reg_x, 0xF);
        validate_argument(reg_y, 0xF);
        if self.cpu_registers[reg_x as usize] == self.cpu_registers[reg_y as usize] {
            self.program_counter += 1;
        }
    }

    // 6XKK
    // Set VX = KK.
    // The interpreter puts the value KK into register VX.
    fn set_register_value(&mut self, reg_x: u8, byte_value: u8)
    {
        validate_argument(reg_x, 0xF);
        validate_argument(byte_value, 0xF);
        self.cpu_registers[reg_x as usize] = byte_value;
    }

    // 7XKK
    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn add(&mut self, reg_x: u8, byte_value: u8)
    {
        validate_argument(reg_x, 0xF);
        validate_argument(byte_value, 0xF);
        self.cpu_registers[reg_x as usize] += byte_value;
    }

    // 8XY0
    // Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    fn load(&mut self, reg_x: u8, reg_y: u8) {
        validate_argument(reg_x, 0xF);
        validate_argument(reg_y, 0xF);
        self.cpu_registers[reg_x as usize] = self.cpu_registers[reg_y as usize];
    }

    // 8XY1
    // Set Vx = Vx OR Vy.
    // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
    fn bit_or(&mut self, reg_x: u8, reg_y: u8) {
        validate_argument(reg_x, 0xF) as usize;
        validate_argument(reg_y, 0xF) as usize;
        self.cpu_registers[reg_x as usize] |= self.cpu_registers[reg_y as usize];
    }

    // 8XY2
    // Set Vx = Vx AND Vy.
    // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
    fn bit_and(&mut self, reg_x: u8, reg_y: u8) {
        validate_argument(reg_x, 0xF) as usize;
        validate_argument(reg_y, 0xF) as usize;
        self.cpu_registers[reg_x as usize] &= self.cpu_registers[reg_y as usize];
    }

    // 8XY3
    // Set Vx = Vx XOR Vy.
    // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    fn bit_xor(&mut self, reg_x: u8, reg_y: u8) {
        validate_argument(reg_x, 0xF) as usize;
        validate_argument(reg_y, 0xF) as usize;
        self.cpu_registers[reg_x as usize] ^= self.cpu_registers[reg_y as usize];
    }

    // 8XY4
    // Set Vx = Vx + Vy, set VF = carry.
    // The values of Vx and Vy are added together.
    // If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
    // Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn add_registers(&mut self, reg_x: u8, reg_y: u8) {
        validate_argument(reg_x, 0xF);
        validate_argument(reg_y, 0xF);

        let reg_x_val: u8 = self.cpu_registers[reg_x as usize];
        let reg_y_val: u8 = self.cpu_registers[reg_y as usize];

        let result = CheckedAdd::checked_add(&reg_x_val, &reg_y_val);

        match result {
            Some(x) => {
                self.cpu_registers[reg_x as usize] = x;
                self.cpu_registers[0xF] = 0;
            }
            None => {
                self.cpu_registers[0xF] = 1;
                self.cpu_registers[reg_x as usize] = (reg_x_val as u16 + reg_y_val as u16) as u8;
            }
        }
    }

    // 8XY5
    // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
    fn sub_registers(&mut self, reg_x: u8, reg_y: u8) {
        validate_argument(reg_x, 0xF);
        validate_argument(reg_y, 0xF);

        let reg_x_val: u8 = self.cpu_registers[reg_x as usize];
        let reg_y_val: u8 = self.cpu_registers[reg_y as usize];

        let result = CheckedSub::checked_sub(&reg_x_val, &reg_y_val);

        match result {
            Some(x) => {
                self.cpu_registers[0xF] = 1;
                self.cpu_registers[reg_x as usize] = x;
            }
            None => {
                self.cpu_registers[0xF] = 0;
                self.cpu_registers[reg_x as usize] = 255 - ((reg_y_val - reg_x_val) - 1)
            }
        }
    }

    // 8XY6
    // Set Vx = Vx SHR 1.
    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0.
    // Then Vx is divided by 2.
    fn shift_right_register(&mut self, reg_x: u8)
    {
        validate_argument(reg_x, 0xF);
        self.cpu_registers[reg_x as usize] = {
            if self.cpu_registers[reg_x as usize] & 1 == 1
            { 1 } else { 0 }
        };
        self.cpu_registers[reg_x as usize] >>= 1;
    }

    // 8XY7
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, then VF is set to 1, otherwise 0.
    // Then Vx is subtracted from Vy, and the results stored in Vx.
    fn sub_registers_not(&mut self, reg_x: u8, reg_y: u8) {
        validate_argument(reg_x, 0xF);
        validate_argument(reg_y, 0xF);

        let reg_x_val: u8 = self.cpu_registers[reg_x as usize];
        let reg_y_val: u8 = self.cpu_registers[reg_y as usize];

        let result = CheckedSub::checked_sub(&reg_y_val, &reg_x_val);

        match result {
            Some(y) => {
                self.cpu_registers[0xF] = 1;
                self.cpu_registers[reg_y as usize] = y;
            }
            None => {
                self.cpu_registers[0xF] = 0;
                self.cpu_registers[reg_y as usize] = 255 - ((reg_x_val - reg_y_val) - 1)
            }
        }
    }

    // 8XYE
    // Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    fn shift_left_register(&mut self, reg_x: u8)
    {
        validate_argument(reg_x, 0xF);
        self.cpu_registers[reg_x as usize] = {
            if self.cpu_registers[reg_x as usize] & 1 == 1
            { 1 } else { 0 }
        };
        self.cpu_registers[reg_x as usize] <<= 1;
    }

    // 9XY0
    // Skip next instruction if Vx != Vy.
    // The values of Vx and Vy are compared, and if they are not equal,
    // the program counter is increased by 2.
    // todo: see 3XKK
    fn skip_next_if_vx_not_eql_vy(&mut self, reg_x: u8, reg_y: u8)
    {
        validate_argument(reg_x, 0xF);
        validate_argument(reg_y, 0xF);
        if self.cpu_registers[reg_x as usize] != self.cpu_registers[reg_y as usize] {
            self.program_counter += 1;
        }
    }

    // ANNN
    // Sets I to the address NNN.
    fn set_index_register(&mut self, value: u16) {
        validate_argument(value, 0x0FFF);
        self.index_register = value;
    }

    // BNNN
    // Jumps to the address NNN plus V0..
    fn jump_to_address_plus_v0(&mut self, value: u16) {
        validate_argument(value, 0xFFF);
        self.program_counter = value + (self.cpu_registers[0] as u16);
    }

    // CXNN
    // Set Vx = random byte AND kk.
    // The interpreter generates a random number from 0 to 255,
    // which is then ANDed with the value kk.
    // The results are stored in Vx.
    fn set_rand(&mut self, reg_x: u8, value: u8) {
        let mut rng = rand::thread_rng();
        let random_num: u8 = rng.gen();
        self.cpu_registers[reg_x as usize] = value & random_num;
    }


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn fetch_cycle_test() {
        let mut c: Chip8 = Chip8::new();
        c.memory[c.program_counter as usize] = 0xA2;
        c.memory[(c.program_counter + 1) as usize] = 0xF0;
        // should execute OPCODE A2F0
        c.fetch_cycle();
        assert_eq!(c.index_register, 0x02F0);
    }

    #[test]
    pub fn clear_screen_test() {
        let mut c: Chip8 = Chip8::new();
        c.gfx[0] = 1;
        c.clear_screen();

        let expected_gfx = [0; 64 * 32];
        assert_eq!(
            c.gfx.len(),
            expected_gfx.len(),
            "Arrays don't have the same length"
        );
        assert!(
            c.gfx.iter().zip(expected_gfx.iter()).all(|(a, b)| a == b),
            "Arrays are not equal"
        );
    }

    #[test]
    pub fn annn_opcode_test() {
        let mut c: Chip8 = Chip8::new();
        c.execute_instruction(0xA123);
        assert_eq!(c.index_register, 0x0123);
    }

    #[test]
    #[should_panic]
    pub fn bad_opcode_test() {
        let mut c: Chip8 = Chip8::new();
        c.execute_instruction(0x69);
    }

    #[test]
    pub fn jump_to_test() {
        let mut c: Chip8 = Chip8::new();
        c.cpu_registers[0] = 0x69;
        c.jump_to(0x0123);
        assert_eq!(c.program_counter, 0x0123 + 0x69);
        c.jump_to(0x0433);
        assert_eq!(c.program_counter, 0x0433 + 0x69);
    }

    #[test]
    pub fn set_index_register_test() {
        let mut c: Chip8 = Chip8::new();
        c.set_index_register(100 as u16);
        assert_eq!(c.index_register, 100);
    }

    #[test]
    pub fn set_rand_test() {
        let mut c: Chip8 = Chip8::new();
        // I don't fucking know how to test this...
    }

    #[test]
    pub fn add_test() {
        let mut c: Chip8 = Chip8::new();
        c.add(10 as u8, 12 as u8);
        assert_eq!(c.cpu_registers[10 as usize], 12);

        c.add(10 as u8, 3 as u8);
        assert_eq!(c.cpu_registers[10 as usize], 15);

        c.add(4 as u8, 3 as u8);
        assert_eq!(c.cpu_registers[4 as usize], 3);
    }

    #[test]
    pub fn add_registers_test() {
        let mut c: Chip8 = Chip8::new();
        c.cpu_registers[0] = 4;
        c.cpu_registers[2] = 3;
        c.cpu_registers[4] = 3;
        c.cpu_registers[5] = 1;
        c.cpu_registers[6] = 0xFF;

        assert_eq!(c.cpu_registers[0xF], 0); // todo: remove this eager test

        c.add_registers(0, 2);
        assert_eq!(c.cpu_registers[0], 7);
        assert_eq!(c.cpu_registers[2], 3);
        assert_eq!(c.cpu_registers[0xF], 0);

        c.add_registers(6, 5);
        assert_eq!(c.cpu_registers[6], 0);
        assert_eq!(c.cpu_registers[0xF], 1);

        c.add_registers(6, 5);
        assert_eq!(c.cpu_registers[6], 1);
        assert_eq!(c.cpu_registers[0xF], 0);
    }

    #[test]
    pub fn sub_registers_test() {
        let mut c: Chip8 = Chip8::new();
        c.cpu_registers[0] = 4;
        c.cpu_registers[1] = 4;
        c.cpu_registers[2] = 3;
        c.cpu_registers[4] = 3;
        c.cpu_registers[5] = 1;
        c.cpu_registers[6] = 0xFF;

        c.sub_registers(0, 2);
        assert_eq!(c.cpu_registers[0], 1);
        assert_eq!(c.cpu_registers[2], 3);
        assert_eq!(c.cpu_registers[0xF], 1);

        c.sub_registers(4, 5);
        assert_eq!(c.cpu_registers[4], 2);
        assert_eq!(c.cpu_registers[5], 1);
        assert_eq!(c.cpu_registers[0xF], 1);

        c.sub_registers(2, 1);
        assert_eq!(c.cpu_registers[1], 4);
        assert_eq!(c.cpu_registers[2], 255);
        assert_eq!(c.cpu_registers[0xF], 0);
    }

    #[test]
    pub fn bit_or_test() {
        let mut c: Chip8 = Chip8::new();
        c.cpu_registers[0] = 4;
        c.cpu_registers[2] = 3;
        c.cpu_registers[4] = 3;
        c.cpu_registers[5] = 1;

        c.bit_or(2 as u8, 0 as u8);
        assert_eq!(c.cpu_registers[2], 7);

        c.bit_or(3 as u8, 4 as u8);
        assert_eq!(c.cpu_registers[3], 3);

        c.bit_or(5 as u8, 4 as u8);
        assert_eq!(c.cpu_registers[5], 3);
    }
}
