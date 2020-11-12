pub struct chip8 {

    // We should break this into cohesive components
    memory: [u8;4096],
    cpu_registers: [u8;16],
    index_register: u16,
    program_counter: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack:stack,
    stack_data:[u16;16],
    sp : u16,
    key_states:u8, 
}

impl chip8 {

    pub fn new() -> chip8 {
        chip8
        {
        }
    }

    pub fn execute_instruction(opcode: u32) {}

    pub mod opcodes {

        use super::*;

        // 7XNN
        // Adds NN to VX. (Carry flag is not changed)
        // Panics if first nibble is not empty
        pub fn add(registerX: u8, valueNN : u8)
        {
            if(registerX >= 16)
                panic!("Register X out of bounds. Value : {}", registerX);

            cpu_registers[register] += valueNN;
        }

    }
}

#[cfg(test)]
mod tests{

    #[test]
    pub fn add_test()
    {
        assert_eq!(2, 2);
    }

}
