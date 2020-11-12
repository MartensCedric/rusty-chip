pub struct chip8 {
    // We should break this into cohesive components
    memory: [u8; 4096],
    cpu_registers: [u8; 16],
    index_register: u16,
    program_counter: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack_data: [u16; 16],
    sp: u16,
    key_states: u8,
}

impl chip8 {
    pub fn new() -> chip8 {
        chip8 {
            memory: [0; 4096],
            cpu_registers: [0; 16],
            index_register: 0,
            program_counter: 0,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack_data: [0; 16],
            sp: 0,
            key_states: 0,
        }
    }

    pub fn execute_instruction(opcode: u32) {}

    // 7XNN
    // Adds NN to VX. (Carry flag is not changed)
    // Panics if first nibble is not empty
    fn add(&mut self, register_x: u8, value_NN: u8) {
        if register_x >= 16 {
            panic!("Register X out of bounds. Value : {}", register_x);
        }
        self.cpu_registers[register_x as usize] += value_NN;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn add_test() {
        assert_eq!(2, 2);
    }
}
