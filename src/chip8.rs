use crate::chip8_util::validate_argument;
use num::CheckedAdd;
use num::CheckedSub;
use rand::Rng;

pub struct Chip8 {
    // We should break this into cohesive components
    memory: [u8; 4096],
    cpu_registers: [u8; 16],
    index_register: u16,
    program_counter: u16,
    pub gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack_data: Vec<u16>,
    key_states: u16,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0; 4096],
            cpu_registers: [0; 16],
            index_register: 0,
            program_counter: 0x200, // CHIP8 expects PC to start at 0x200
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack_data: vec![0; 16],
            key_states: 0,
        }
    }

    pub fn fetch_cycle(&mut self) {
        let opcode: u16 = self.fetch_next();
        println!("Executing opcode: {:#X}", opcode);
        self.execute_instruction(opcode);
    }

    pub fn init_memory(&mut self, read_only_memory: &[u8], start_index: usize) {
        let rom_length: usize = read_only_memory.len();
        for i in start_index..(start_index + rom_length) {
            self.memory[i as usize] = read_only_memory[(i - start_index) as usize];
        }
    }

    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn is_sound_active(&self) -> bool {
        self.sound_timer > 0
    }

    // Executes the given opcode
    // Includes decoding and executing the given opcode
    fn execute_instruction(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x000 => (), // Used for old machines, do nothing here.
                0x0E0 => self.clear_screen(),
                0x0EE => self.subroutine_return(),
                _ => panic!("Unknown opcode: {}", opcode),
            },
            0x1000 => self.jump_to_address(opcode & 0x0FFF),
            0x2000 => self.call_address(opcode & 0x0FFF),
            0x3000 => self.skip_next_if_byte_is_vx(
                ((opcode & 0x0F00) >> 8) as u8, // XKK
                (opcode & 0x0FF) as u8,
            ),
            0x4000 => self.skip_next_if_byte_is_not_vx(
                ((opcode & 0x0F00) >> 8) as u8, // XKK
                (opcode & 0x0FF) as u8,
            ),
            0x5000 => self.skip_next_if_vx_eql_vy(
                ((opcode & 0xF00) >> 8) as u8, // XY0
                ((opcode & 0x0F0) >> 4) as u8,
            ),
            0x6000 => self.set_register_value(
                // XKK
                ((opcode & 0xF00) >> 8) as u8,
                (opcode & 0x0FF) as u8,
            ),
            0x7000 => self.add(
                // XKK
                ((opcode & 0xF00) >> 8) as u8,
                (opcode & 0x0FF) as u8,
            ),
            0x8000 => match opcode & 0xF00F {
                0x8000 => self.load(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                ),
                0x8001 => self.bit_or(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                ),
                0x8002 => self.bit_and(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                ),
                0x8003 => self.bit_xor(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                ),
                0x8004 => self.add_registers(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                ),
                0x8005 => self.sub_registers(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                ),
                0x8006 => self.shift_right_register(((opcode & 0x0F00) >> 8) as u8),
                0x8007 => self.sub_registers_not(
                    ((opcode & 0x0F00) >> 8) as u8,
                    ((opcode & 0x00F0) >> 4) as u8,
                ),
                0x800E => self.shift_left_register(((opcode & 0x0F00) >> 8) as u8),
                _ => panic!("Unknown opcode: {}", opcode),
            },
            0x9000 => self.skip_next_if_vx_not_eql_vy(
                ((opcode & 0x0F00) >> 8) as u8,
                ((opcode & 0x00F0) >> 4) as u8,
            ),
            0xA000 => self.set_index_register(opcode & 0x0FFF),
            0xB000 => self.jump_to_address_plus_v0(opcode & 0x0FFF),
            0xC000 => self.set_rand(((opcode & 0x0F00) >> 8) as u8, (opcode & 0x0FF) as u8),
            0xD000 => self.draw(
                ((opcode & 0x0F00) >> 8) as u8,
                ((opcode & 0x00F0) >> 4) as u8,
                (opcode & 0xF) as u8,
            ),
            0xE000 => match opcode & 0xF0FF {
                0xE09E => self.skip_if_key_down(((opcode & 0x0F00) >> 8) as u8),
                0xE0A1 => self.skip_if_key_up(((opcode & 0x0F00) >> 8) as u8),
                _ => panic!("Unknown opcode: {}", opcode),
            },
            0xF000 => match opcode & 0xF0FF {
                0xF007 => self.read_delay_timer(((opcode & 0x0F00) >> 8) as u8),
                0xF00A => self.wait_for_key(((opcode & 0x0F00) >> 8) as u8),
                0xF015 => self.set_delay_timer(((opcode & 0x0F00) >> 8) as u8),
                0xF018 => self.set_sound_timer(((opcode & 0x0F00) >> 8) as u8),
                0xF01E => self.index_reg_add(((opcode & 0x0F00) >> 8) as u8),
                0xF029 => self.set_index_to_character_address(((opcode & 0x0F00) >> 8) as u8),
                0xF033 => self.store_bcd(((opcode & 0x0F00) >> 8) as u8),
                0xF055 => self.store_registers(((opcode & 0x0F00) >> 8) as u8),
                0xF065 => self.read_memory(((opcode & 0x0F00) >> 8) as u8),
                _ => panic!("Unknown opcode: {}", opcode),
            },
            _ => {
                panic!("Unknown opcode: {}", opcode);
            }
        }
    }

    // essentially combine PC: u8 and PC+1: u8 into one u16 opcode to execute using bitshift ops
    fn fetch_next(&mut self) -> u16 {
        let opcode: u16 = (self.memory[self.program_counter as usize] as u16) << 8
            | self.memory[(self.program_counter + 1) as usize] as u16;

        self.program_counter += 2;
        opcode
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
            }
            None => panic!("Nothing in the stack!"),
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
    fn call_address(&mut self, address: u16) {
        self.stack_data.push(self.program_counter as u16);
        self.program_counter = validate_argument(address, 0x0FFF);
    }

    // 3XKK
    // Skip next instruction if VX = KK.
    // The interpreter compares register Vx to kk, and if they are equal,
    // increments the program counter by 2.
    fn skip_next_if_byte_is_vx(&mut self, reg_x: u8, byte_value: u8) {
        validate_argument(reg_x, 0xF);
        validate_argument(byte_value, 0xFF);
        if self.cpu_registers[reg_x as usize] == byte_value {
            self.program_counter += 2;
        }
    }

    // 4XKK
    // Skip next instruction if Vx != kk.
    // The interpreter compares register Vx to kk,
    // and if they are not equal, increments the program counter by 2.
    fn skip_next_if_byte_is_not_vx(&mut self, reg_x: u8, byte_value: u8) {
        validate_argument(reg_x, 0xF);
        validate_argument(byte_value, 0xFF);
        if self.cpu_registers[reg_x as usize] != byte_value {
            self.program_counter += 2;
        }
    }

    // 5XY0
    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal,
    // increments the program counter by 2.
    fn skip_next_if_vx_eql_vy(&mut self, reg_x: u8, reg_y: u8) {
        validate_argument(reg_x, 0xF);
        validate_argument(reg_y, 0xF);
        if self.cpu_registers[reg_x as usize] == self.cpu_registers[reg_y as usize] {
            self.program_counter += 2;
        }
    }

    // 6XKK
    // Set VX = KK.
    // The interpreter puts the value KK into register VX.
    fn set_register_value(&mut self, reg_x: u8, byte_value: u8) {
        validate_argument(reg_x, 0xF);
        validate_argument(byte_value, 0xFF);
        self.cpu_registers[reg_x as usize] = byte_value;
    }

    // 7XKK
    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn add(&mut self, reg_x: u8, byte_value: u8) {
        validate_argument(reg_x, 0xF);
        validate_argument(byte_value, 0xFF);

        let mut result: u16 = byte_value as u16 + self.cpu_registers[reg_x as usize] as u16;
        if result > 255 {
            result -= 255;
        }
        self.cpu_registers[reg_x as usize] = result as u8;
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
    fn shift_right_register(&mut self, reg_x: u8) {
        validate_argument(reg_x, 0xF);
        self.cpu_registers[reg_x as usize] = {
            if self.cpu_registers[reg_x as usize] & 1 == 1 {
                1
            } else {
                0
            }
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
    fn shift_left_register(&mut self, reg_x: u8) {
        validate_argument(reg_x, 0xF);
        self.cpu_registers[reg_x as usize] = {
            if self.cpu_registers[reg_x as usize] & 1 == 1 {
                1
            } else {
                0
            }
        };
        self.cpu_registers[reg_x as usize] <<= 1;
    }

    // 9XY0
    // Skip next instruction if Vx != Vy.
    // The values of Vx and Vy are compared, and if they are not equal,
    // the program counter is increased by 2.
    fn skip_next_if_vx_not_eql_vy(&mut self, reg_x: u8, reg_y: u8) {
        validate_argument(reg_x, 0xF);
        validate_argument(reg_y, 0xF);
        if self.cpu_registers[reg_x as usize] != self.cpu_registers[reg_y as usize] {
            self.program_counter += 2;
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

    // DXYN
    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    // The interpreter reads n bytes from memory, starting at the address stored in I.
    // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
    // Sprites are XORed onto the existing screen.
    // If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
    // If the sprite is positioned so part of it is outside the coordinates of the display,
    // it wraps around to the opposite side of the screen.
    fn draw(&mut self, reg_x: u8, reg_y: u8, bytes_to_read: u8) {
        validate_argument(reg_x, 0xFF);
        validate_argument(reg_y, 0xFF);

        let x: u8 = self.cpu_registers[reg_x as usize];
        let y: u8 = self.cpu_registers[reg_y as usize];

        let reading_address: u16 = self.index_register;

        let mut pixel_was_erased: bool = false;
        for i in 0..bytes_to_read {
            let mut y_wrapped: u16 = y as u16 + i as u16;
            y_wrapped %= 32;

            if self.draw_byte(
                x,
                y_wrapped as u8,
                self.memory[(reading_address + i as u16) as usize],
            ) {
                pixel_was_erased = true;
            }
        }

        self.cpu_registers[0xF] = if pixel_was_erased { 1 } else { 0 };
    }

    // Draws byte
    // Wraps around if needed
    // Returns true if it cleared a pixel
    fn draw_byte(&mut self, x: u8, y: u8, byte: u8) -> bool {
        let mut pixel_was_erased = false;
        let index: usize = ((y as usize) * 64 + (x as usize)) as usize;
        println!(
            "Drawing byte {:#X} at ({},{}), this is index {}",
            byte, x, y, index
        );
        for i in 0..8 {
            let pixel: u8 = self.gfx[index + i];
            self.gfx[index + i] ^= if ((byte >> (7 - i)) & 1) == 1 { 255 } else { 0 };

            if pixel == 255 && self.gfx[index + i] != 255 {
                pixel_was_erased = true;
            }
        }

        pixel_was_erased
    }

    // EX9E
    // Skip next instruction if key with the value of Vx is pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in
    // the down position, PC is increased by 2.
    fn skip_if_key_down(&mut self, key: u8) {
        let is_key_pressed = false;
        if is_key_pressed {
            self.program_counter += 2;
        }
    }

    // EXA1
    // Skip next instruction if key with the value of Vx is not pressed.
    // Checks the keyboard, and if the key corresponding to the value of Vx is currently in
    // the up position, PC is increased by 2.
    fn skip_if_key_up(&mut self, key: u8) {
        let is_key_pressed = false;
        if !is_key_pressed {
            self.program_counter += 2;
        }
    }

    // FX07
    // Set Vx = delay timer value.
    // The value of DT is placed into Vx.
    fn read_delay_timer(&mut self, reg_x: u8) {
        validate_argument(reg_x, 0xFF);
        self.cpu_registers[reg_x as usize] = self.delay_timer;
    }

    // FX0A
    // Wait for a key press, store the value of the key in Vx.
    // All execution stops until a key is pressed, then the value of that key is stored in Vx.
    fn wait_for_key(&mut self, reg_x: u8) {
        validate_argument(reg_x, 0xFF);
        panic!("wait_for_key not implemented!");
        let key_pressed = 0;
        self.cpu_registers[reg_x as usize] = key_pressed;
    }

    // FX15
    // Set delay timer = Vx.
    // DT is set equal to the value of Vx.
    fn set_delay_timer(&mut self, reg_x: u8) {
        validate_argument(reg_x, 0xFF);
        self.delay_timer = self.cpu_registers[reg_x as usize];
    }

    // FX18
    // Set sound timer = Vx.
    // ST is set equal to the value of Vx.
    fn set_sound_timer(&mut self, reg_x: u8) {
        validate_argument(reg_x, 0xFF);
        self.sound_timer = self.cpu_registers[reg_x as usize];
    }

    // FX1E
    // Set I = I + Vx.
    // The values of I and Vx are added, and the results are stored in I.
    fn index_reg_add(&mut self, reg_x: u8) {
        validate_argument(reg_x, 0xFF);
        self.index_register = self.cpu_registers[reg_x as usize] as u16;
    }

    // FX29
    // Set I = location of sprite for digit Vx.
    // The value of I is set to the location for the hexadecimal sprite corresponding
    // to the value of Vx.
    // This points to the reserved memory from the file read_only_memory.dat
    fn set_index_to_character_address(&mut self, value: u8) {
        validate_argument(value, 0xF);
        let address: u16 = (value * 5) as u16;
        self.index_register = address;
    }

    // FX33
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    // The interpreter takes the decimal value of Vx, and places the hundreds digit in memory
    // at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    fn store_bcd(&mut self, reg_x: u8) {
        validate_argument(reg_x, 0xFF);
        let value: u8 = self.cpu_registers[reg_x as usize];
        let hundreds: u8 = value / 100;
        let tens: u8 = (value - hundreds * 100) / 10;
        let digits: u8 = (value - hundreds * 100) - tens * 10;

        let index: usize = self.index_register as usize;
        self.memory[index] = hundreds;
        self.memory[index + 1] = tens;
        self.memory[index + 2] = digits;
    }

    // FX55
    // Store registers V0 through Vx in memory starting at location I.
    // The interpreter copies the values of registers V0 through Vx into memory,
    // starting at the address in I.
    fn store_registers(&mut self, value: u8) {
        validate_argument(value, 0xF);
        for i in 0..(value + 1) {
            let index = i as usize;
            let memory_location = (self.index_register as usize + index) as usize;
            self.memory[memory_location] = self.cpu_registers[index];
        }
    }

    // FX65
    // Read registers V0 through Vx from memory starting at location I.
    // The interpreter reads values from memory starting at location I into registers V0 through Vx.
    fn read_memory(&mut self, value: u8) {
        validate_argument(value, 0xF);
        for i in 0..value {
            let index = i as usize;
            let memory_location = (self.index_register as usize + index) as usize;
            self.cpu_registers[index] = self.memory[memory_location];
        }
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
        c.fetch_cycle();
        assert_eq!(c.index_register, 0x02F0);
    }

    #[test]
    pub fn draw_test() {
        let mut c: Chip8 = Chip8::new();
        let mut gfx: [u8; 64 * 32] = [0; 64 * 32];

        assert_eq!(
            gfx.len(),
            c.gfx.len(),
            "Arrays do not have the same length! {} vs {}",
            gfx.len(),
            c.gfx.len()
        );
        assert!(
            gfx.iter().zip(gfx.iter()).all(|(a, b)| a == b),
            "Arrays are not equal"
        );

        c.memory[0x300] = 0xFF;
        c.memory[0x301] = 0x55;

        c.execute_instruction(0x60FF);
        c.execute_instruction(0x6155);
        c.execute_instruction(0xA300);
        c.execute_instruction(0xF155);
        c.execute_instruction(0x6000);
        c.execute_instruction(0x6100);
        c.execute_instruction(0xD012);

        for i in 0..8 {
            gfx[i] = 0xFF;
        }

        gfx[64] = 0;
        gfx[65] = 0xFF;
        gfx[66] = 0;
        gfx[67] = 0xFF;
        gfx[68] = 0;
        gfx[69] = 0xFF;
        gfx[70] = 0;
        gfx[71] = 0xFF;

        assert_eq!(
            gfx.len(),
            c.gfx.len(),
            "Arrays do not have the same length! {} vs {}",
            gfx.len(),
            c.gfx.len()
        );
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
    pub fn load_index_test() {
        let mut c: Chip8 = Chip8::new();
        c.execute_instruction(0xA123);
        assert_eq!(c.index_register, 0x0123);
    }

    #[test]
    #[should_panic]
    pub fn bad_opcode_test() {
        let mut c: Chip8 = Chip8::new();
        c.execute_instruction(0x68);
    }

    #[test]
    pub fn jump_to_address_plus_v0_test() {
        let mut c: Chip8 = Chip8::new();
        c.cpu_registers[0] = 0x68;
        c.jump_to_address_plus_v0(0x0123);
        assert_eq!(c.program_counter, 0x0123 + 0x68);
        c.jump_to_address_plus_v0(0x0433);
        assert_eq!(c.program_counter, 0x0433 + 0x68);
    }

    #[test]
    pub fn set_index_register_test() {
        let mut c: Chip8 = Chip8::new();
        c.set_index_register(100 as u16);
        assert_eq!(c.index_register, 100);
    }

    #[test]
    pub fn set_index_to_character_address_test() {
        let mut c: Chip8 = Chip8::new();
        c.set_index_to_character_address(0);
        assert_eq!(c.index_register, 0);

        c.set_index_to_character_address(1);
        assert_eq!(c.index_register, 5);

        c.set_index_to_character_address(2);
        assert_eq!(c.index_register, 10);
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
