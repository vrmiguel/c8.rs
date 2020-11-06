/// The fontset for the CHIP-8.
/// Every character is 4 pixels wide and 5 pixels tall.
const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(PartialEq)]
/// Used by comparison opcodes
enum ComparisonType {
    Equality,
    Inequality,
}

// Allow non-snake-case naming of variables I and V.
#[allow(non_snake_case)]
#[allow(dead_code)]
/// Represents the CHIP-80 virtual machine.
pub struct VirtualMachine {
    // Holds an operation code (two bytes)
    opcode: u16,

    // Represents the Chip-8 stack
    pub stack: [u8; 16],

    // Stack pointer
    pub sp: u16,

    /* Represents the 4KB of memory that
    the CHIP-8 has. */
    memory: [u8; 4096],

    /* CPU registers:
       15 general purpose registers (V0, V1, ..., VE)
       A sixteenth register is used for carry-one operations.
    */
    V: [u8; 16],

    // Index register
    I: u16,

    // Program counter
    pc: u16,

    // The CHIP-8 has a 64 x 32 screen
    // The `graphics` array holds the state of every pixel
    // If true, the pixel is white.
    graphics: [bool; 64 * 32],

    // If true, the contents of `graphics` will be drawn to screen
    draw_to_screen: bool,

    // The CHIP-8 supports 16 keys (hex-based)
    // `keypad` holds the current state of the keypad
    keypad: [u8; 16],
}

#[allow(dead_code)]
impl VirtualMachine {
    /// Creates and initializes all the variables within the virtual machine
    pub fn new() -> VirtualMachine {
        let mut vm = VirtualMachine {
            opcode: 0,
            I: 0,
            sp: 0,
            // The program counter starts at 0x200
            pc: 0x200,
            // Fill the stack with zeroes
            stack: [0; 16],
            // Clean the keypad state
            keypad: [0; 16],
            // Fill the memory with zeroes
            memory: [0; 4096],
            // Clear display (all black)
            graphics: [false; 64 * 32],
            // Clear registers
            V: [0; 16],
            // There's nothing to draw to screen yet
            draw_to_screen: false,
        };

        // Load the fontset into memory
        for i in 0..=79 {
            vm.memory[i] = FONTSET[i];
        }

        vm
    }

    /// Reads a new opcode from memory
    fn fetch_opcode(&self) -> u16 {
        let first_byte = (self.memory[self.pc as usize] as u16) << 8; // Cast the memory position to u16 to avoid arith. overflow
        let second_byte = (self.memory[self.pc as usize + 1_usize]) as u16;
        first_byte | second_byte
    }

    /// Clears the CHIP-80 screen
    fn clear_screen(&mut self) {
        self.graphics = [false; 64 * 32];
        self.draw_to_screen = true;
    }

    fn draw_sprite(&mut self) {
        let x_idx = (self.opcode & 0x0F00) >> 8;
        let y_idx = (self.opcode & 0x00F0) >> 4;
        let x = self.V[x_idx as usize];
        let y = self.V[y_idx as usize];

        // The height of the sprite
        let height = self.opcode & 0x000F;

        // Reset VF
        self.V[0xF as usize] = 0;

        for yline in 0..height {
            // Get the pixel vaue from the memory starting at I
            let pixel = self.memory[(self.I + yline) as usize];
            // Loop over the 8 bits of the current row
            for xline in 0..8 {
                if pixel & (0x80 >> xline) != 0 {
                    let pos = (x + xline + ((y + yline as u8) * 64)) as usize;
                    if self.graphics[pos] {
                        self.V[0xF as usize] = 1;
                    }
                    self.graphics[pos] = !self.graphics[pos];
                }
            }
        }

        self.draw_to_screen = true;
    }

    #[allow(non_snake_case)]
    /// Skips an instruction dependending on if VX and NN are equal (or unequal). 
    /// Used by opcodes 3XNN and 4XNN.
    /// `cmptype` defines whether to skip an instruction if VX == N or if VX != N.
    fn compare_vx_and_nn(&mut self, cmptype: ComparisonType) {
        let X = (self.opcode & 0x0F00) >> 8;
        let VX = self.V[X as usize] as u16;
        let NN = self.opcode & 0x00FF;
        if cmptype == ComparisonType::Equality 
        {
            // Compare if VX == NN
            if VX == NN {
                // VX == NN, so we skip the next instruction
                self.pc += 4;
            } else {
                self.pc += 2;
            }
        } else {
            // Compare if VX != NN
            if VX != NN {
                // VX != NN, so we skip the next instruction
                self.pc += 4;
            } else {
                self.pc += 2;
            }
        }
    }

    #[allow(non_snake_case)]
    fn run_cycle(&mut self) {
        self.opcode = self.fetch_opcode();
        match self.opcode & 0xF000 {
            0x0000 => {
                /* Opcode's first byte is null, so
                we must now only compare its last byte. */
                match self.opcode & 0x000F {
                    0x0000 => {
                        // Opcode 0x00E0: Clears the screen
                        self.clear_screen();
                        self.pc += 2;
                    }

                    0x000E => {
                        // Opcode 0x00EE: Returns from subroutine
                        self.sp -= 1;
                        let new_program_counter = self.stack[self.sp as usize];
                        self.pc = new_program_counter as u16 + 2;
                    }

                    op @ _ => {
                        eprintln!("Unknown opcode [0x0000#04x{}]", op);
                    }
                }
            }

            0x1000 => {
                // Opcode 0x1000: Jumps to address NNN
                self.pc = self.opcode & 0x0FFF;
            }

            0x2000 => {
                // Opcode 2NNN: Calls subroutine located at NNN
                // TODO: make sure that `self.pc as u8` can't overflow
                self.stack[self.sp as usize] = self.pc as u8;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }

            0x3000 => {
                // Opcode 3XNN: Skips the next instruction if VX == NN.
                self.compare_vx_and_nn(ComparisonType::Equality);
            }

            0x4000 => {
                // Opcode 4XNN: Skips the next instruction if VX != NN.
                self.compare_vx_and_nn(ComparisonType::Inequality);
            }

            0x5000 => {
                // Opcode 5XY0: Skips the next instruction if VX == VY
                let X = (self.opcode & 0x0F00) >> 8;
                let VX = self.V[X as usize] as u16; 
                let Y = (self.opcode & 0x00F0) >> 4;
                let VY = self.V[Y as usize] as u16;      
                if VX == VY {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            0x6000 => {
                // Opcode 6XNN: sets VX to NN
                let X  = (self.opcode & 0x0F00) >> 8;
                let NN = (self.opcode & 0x00FF) as u8;
                self.V[X as usize] = NN;
                self.pc += 2;
            }

            0xD000 => {
                /*  Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
                Each row of 8 pixels is read as bit-coded starting from memory location I.
                The I value doesn’t change after the execution of this instruction.
                As described above, VF is set to 1 if any screen pixels are flipped from set to unset when
                the sprite is drawn, and to 0 if that doesn’t happen. */
                self.draw_sprite();
                self.pc += 2;
            }

            op @ _ => {
                eprintln!("Unknown opcode #08x{}", op);
            }
        }
    }
}
