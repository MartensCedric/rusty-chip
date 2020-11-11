pub struct chip8 {
    memory: [u8;4096],
    cpu_registers: [u8;16],
    index_register: u16,
    program_counter: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
}

impl chip8 {
    pub fn execute_instruction(opcode: u8, arg1: u8, arg2: u8, arg3: u8) {}

    pub mod opcodes {
        use super::*;
        mod tests {
            use super::*;
        }
    }
}
