use rand::Rng;
use ux::*;

pub struct Chip8 {
    // We should break this into cohesive components
    memory: [u8; 4096],
    cpu_registers: [u8; 16],
    opcode: u8,
    index_register: u16,
    program_counter: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack_data: [u16; 16],
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
            stack_data: [0; 16],
            sp: 0,
            key_states: 0,
        }
    }

    pub fn fetch_cycle(&mut self) {
        // fetch opcode
        // essentially combine PC: u8 and PC+1: u8 into one u16 opcode to execute using bitshift ops
        let opcode: u16 = (self.memory[self.program_counter as usize] as u16) << 8
            | self.memory[(self.program_counter + 1) as usize] as u16;
        // execute opcode
        self.execute_instruction(opcode);
    }

    // Executes the given opcode
    // Includes decoding and executing the given opcode
    // TODO: complete this to handle every single opcode and call
    // the correct assosiated function
    pub fn execute_instruction(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0xA000 => self.set_i(opcode & 0x0FFF),
            _ => {
                panic!("Unknown opcode!");
            }
        }
    }

    // 00E0
    // Clears the screen.
    fn clear_screen(&mut self) {
        self.gfx = [0; 64 * 32];
    }

    // ANNN
    // Sets I to the address NNN.
    fn set_i(&mut self, value_nnn: u16) {
        self.index_register = 0x0FFF & value_nnn;
    }

    // BNNN
    // Jumps to the address NNN plus V0..
    fn jump_to(&mut self, value_nnn: u16) {
        self.program_counter = (0x0FFF & value_nnn) + (self.cpu_registers[0] as u16);
    }

    // CXNN
    // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
    fn set_rand(&mut self, index: u8, value_nn: u8) {
        let mut rng = rand::thread_rng();
        let random_num: u8 = rng.gen();
        self.cpu_registers[index as usize] = value_nn & random_num;
    }

    // 7XNN
    // Adds NN to VX. (Carry flag is not changed)
    // Panics if registerX is out of bounds
    fn add(&mut self, register_x: u8, value_nn: u8) {
        self.cpu_registers[register_x as usize] += value_nn;
    }

    // 8XY1
    // Sets VX to VX or VY. (Bitwise OR operation)
    // Panics if registers are out of bounds
    fn bit_or(&mut self, register_x: u8, register_y: u8) {
        self.cpu_registers[register_x as usize] |= self.cpu_registers[register_y as usize];
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
        assert_eq!(c.gfx, [0; 64 * 32]);
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
    }

    #[test]
    pub fn set_i_test() {
        let mut c: Chip8 = Chip8::new();
        c.set_i(100 as u16);
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
    pub fn bit_or_test() {
        let mut c: Chip8 = Chip8::new();
        c.cpu_registers[0 as usize] = 4;
        c.cpu_registers[2 as usize] = 3;
        c.cpu_registers[4 as usize] = 3;
        c.cpu_registers[5 as usize] = 1;

        c.bit_or(2 as u8, 0 as u8);
        assert_eq!(c.cpu_registers[2 as usize], 7);

        c.bit_or(3 as u8, 4 as u8);
        assert_eq!(c.cpu_registers[3 as usize], 3);

        c.bit_or(5 as u8, 4 as u8);
        assert_eq!(c.cpu_registers[5 as usize], 3);
    }
}
