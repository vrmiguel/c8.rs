use crate::rom::Cartridge;
use std::fmt;
use rand::Rng;

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

const SCREEN_WIDTH: usize  = 64;
const SCREEN_HEIGHT: usize = 32;

#[derive(PartialEq)]
/// Used by comparison opcodes
enum ComparisonType {
    Equality,
    Inequality,
}

/// Used by opcodes 8XY0, 8XY1 and 8XY2,
/// in the context of binary operations between
/// VX and VY.
enum BinOp {
    // VX |= VY
    Or,
    // VX &= VY
    And,
    // VX ^= VY
    Xor,
    // VX = VY
    Attrib,
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
    // pub graphics: [u8; 64 * 32],
    pub graphics: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],

    // If true, the contents of `graphics` will be drawn to screen
    pub draw_to_screen: bool,

    // The CHIP-8 supports 16 keys (hex-based)
    // `keypad` holds the current state of the keypad
    keypad: [u8; 16],

    // General timer register
    delay_timer: u8,

    // `sound_timer` is the buzzer's timer
    // The buzzer sounds whenever this timer reaches zero
    sound_timer: u8
}

impl fmt::Display for VirtualMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OP: {:#04x}, PC: {:#04x}, I: {:#04x}\n", self.opcode, self.pc, self.I)
    }
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
            graphics: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            // Clear registers
            V: [0; 16],
            // There's nothing to draw to screen yet
            draw_to_screen: false,
            // Reset timers
            sound_timer: 0,
            delay_timer: 0
        };

        // Load the fontset into memory
        for (i, &byte) in FONTSET.iter().enumerate() {
            // println!("FONTSET[{}] = {}", i, byte);
            vm.memory[i] = byte;
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
        self.graphics = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.draw_to_screen = true;
    }

    #[allow(non_snake_case)]
    /// Returns the current values of X and VX
    fn vx(&mut self) -> (u8, u8) {
        let X = (self.opcode & 0x0F00) >> 8;
        (X as u8, self.V[X as usize])
    }

    #[allow(non_snake_case)]
    /// Returns the current values of Y and VY
    fn vy(&mut self) -> (u8, u8) {
        let Y = (self.opcode & 0x00F0) >> 4;
        (Y as u8, self.V[Y as usize])
    }

    /// Returns both the current VX and the current VY
    fn vx_vy(&mut self) -> ((u8, u8), (u8, u8)) {
        (self.vx(), self.vy())
    }

    #[allow(non_snake_case)]
    /// Executes a binary operation between VX and VY and attributes it to VX.
    fn vx_vy_bin_op(&mut self, binop: BinOp) {
        // let X = (self.opcode & 0x0F00) >> 8;
        // let Y = (self.opcode & 0x00F0) >> 4;
        // let VY = self.V[Y as usize];
        let ((X, _), (_, VY)) = self.vx_vy();
        match binop {
            BinOp::Attrib => {
                self.V[X as usize] = VY;
            }

            BinOp::Xor => {
                self.V[X as usize] ^= VY;
            }

            BinOp::And => {
                self.V[X as usize] &= VY;
            }

            BinOp::Or => {
                self.V[X as usize] |= VY;
            }
        }
        self.pc += 2;
    }

    #[allow(non_snake_case)]
    /// Skips an instruction dependending on if VX and NN are equal (or unequal).
    /// Used by opcodes 3XNN and 4XNN.
    /// `cmptype` defines whether to skip an instruction if VX == N or if VX != N.
    fn compare_vx_and_nn(&mut self, cmptype: ComparisonType) {
        // let X = (self.opcode & 0x0F00) >> 8;
        // let VX = self.V[X as usize] as u16;
        let (_, VX) = self.vx();
        let NN = (self.opcode & 0x00FF) as u8;
        if cmptype == ComparisonType::Equality {
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
    fn draw_sprite(&mut self) {

        // x := The contents of VX, where X is specified by the current opcode
        // y := The contents of VY, where Y is specified by the current opcode
        let ((_, x), (_, y)) = self.vx_vy();
        let n = (self.opcode & 0x000F) as u8;

        // Reset VF
        self.V[0xF as usize] = 0;

        for byte in 0..(n as usize) {
            // Wrap around if overflown
            let y = (self.V[y as usize] as usize + byte) % SCREEN_HEIGHT;
            for bit in 0..8 {
                let x = (self.V[x as usize] as usize + bit) % SCREEN_WIDTH;
                let I = self.I as usize;
                let color = (self.memory[I + byte] >> (7 - bit)) & 1;
                self.V[0x0F] |= color & self.graphics[y][x];
                self.graphics[y][x] ^= color;
            }
        }

        self.draw_to_screen = true;
    }

    pub fn load_rom(& mut self, cart: Cartridge)
    {
        for i in 0..cart.size {
            self.memory[(i+512) as usize] = cart.data[i as usize];
        }
    }

    #[allow(non_snake_case)]
    pub fn run_cycle(&mut self) {
        self.opcode = self.fetch_opcode();
        match self.opcode & 0xF000 {
            0x0000 => {
                /* Opcode's first byte is null, so
                we must now only compare its last byte. */
                match self.opcode & 0x000F {
                    0x0000 => {
                        p!(:"Opcode 00E0: Clears the screen");
                        // Opcode 00E0: Clears the screen
                        self.clear_screen();
                        self.pc += 2;
                    }

                    0x000E => {
                        p!(:"Opcode 0EE: Returns from subroutine");
                        // Opcode 0EE: Returns from subroutine
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
                p!(:"Opcode 1NNN: Jumps to address NNN");
                // Opcode 1NNN: Jumps to address NNN
                self.pc = self.opcode & 0x0FFF;
            }

            0x2000 => {
                p!(:"Opcode 2NNN: Calls subroutine located at NNN");
                // Opcode 2NNN: Calls subroutine located at NNN
                // TODO: make sure that `self.pc as u8` can't overflow
                self.stack[self.sp as usize] = self.pc as u8;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }

            0x3000 => {
                p!(:"Opcode 3XNN: Skips the next instruction if VX == NN.");
                // Opcode 3XNN: Skips the next instruction if VX == NN.
                self.compare_vx_and_nn(ComparisonType::Equality);
            }

            0x4000 => {
                p!(:"Opcode 4XNN: Skips the next instruction if VX != NN.");
                // Opcode 4XNN: Skips the next instruction if VX != NN.
                self.compare_vx_and_nn(ComparisonType::Inequality);
            }

            0x5000 => {
                p!(:"Opcode 5XY0: Skips the next instruction if VX == VY");
                // Opcode 5XY0: Skips the next instruction if VX == VY
                // let X = (self.opcode & 0x0F00) >> 8;
                // let VX = self.V[X as usize] as u16;
                // let Y = (self.opcode & 0x00F0) >> 4;
                // let VY = self.V[Y as usize] as u16;
                let ((_, VX), (_, VY)) = self.vx_vy();
                if VX == VY {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            0x6000 => {
                p!(:"Opcode 6XNN: sets VX to NN");
                // Opcode 6XNN: sets VX to NN
                // let X  = (self.opcode & 0x0F00) >> 8;
                let (X, _) = self.vx();
                let NN = (self.opcode & 0x00FF) as u8;
                self.V[X as usize] = NN;
                self.pc += 2;
            }

            0x7000 => {
                p!(:"Opcode 7XNN: Adds NN to VX.");
                // Opcode 7XNN: Adds NN to VX.
                // let X  = (self.opcode & 0x0F00) >> 8;
                let (X, VX) = self.vx();
                let mut NN = (self.opcode & 0x00FF) as u16;
                NN = if NN + (VX as u16) > 255 {
                    // Wrap around if overflown
                    NN % 256
                } else {
                    NN                    
                };

                self.V[X as usize] += NN as u8;
                self.pc += 2;
            }

            0x8000 => {
                match self.opcode & 0x000F {
                    0x0000 => {
                        p!(:"Opcode 8XY0: Sets VX to the value of VY");
                        // Opcode 8XY0: Sets VX to the value of VY
                        self.vx_vy_bin_op(BinOp::Attrib);
                    }

                    0x0001 => {
                        p!(:"Opcode 8XY1: Sets VX to (VX | VY)");
                        // Opcode 8XY1: Sets VX to (VX | VY)
                        self.vx_vy_bin_op(BinOp::Or);
                    }

                    0x0002 => {
                        p!(:"Opcode 8XY2: Sets VX to (VX & VY)");
                        // Opcode 8XY2: Sets VX to (VX & VY)
                        self.vx_vy_bin_op(BinOp::And);
                    }

                    0x0003 => {
                        p!(:"Opcode 8XY3: Sets VX to (VX ^ VY)");
                        // Opcode 8XY3: Sets VX to (VX ^ VY)
                        self.vx_vy_bin_op(BinOp::Xor);
                    }

                    0x0004 => {
                        p!(:"Opcode 8XY4: Adds VY to VX.");
                        // Opcode 8XY4: Adds VY to VX. An overflow flag is set if VX + VY > 255
                        // let X = (self.opcode & 0x0F00) >> 8;
                        // let VX = self.V[X as usize] as u16;
                        // let Y = (self.opcode & 0x00F0) >> 4;
                        // let VY = self.V[Y as usize] as u16;

                        let ((X, VX), (_, VY)) = self.vx_vy();
                        let sum = (VX + VY) as u16;
                        if sum > 0xFF {
                            self.V[0xF as usize] = 1;
                        } else {
                            self.V[0xF as usize] = 0;
                        }
                        self.V[X as usize] = (sum & 0xFF) as u8;
                        self.pc += 2;
                    }

                    0x0005 => {
                        p!(:"Opcode 8XY5: Subtracts VY from VX.");
                        // Opcode 8XY5: Subtracts VY from VX.
                        // VF is set when there's been a borrow.
                        // let X = (self.opcode & 0x0F00) >> 8;
                        // let VX = self.V[X as usize] as u16;
                        // let Y = (self.opcode & 0x00F0) >> 4;
                        // let VY = self.V[Y as usize] as u16;
                        let ((X, VX), (_, VY)) = self.vx_vy();
                        // Set the borrow flag
                        self.V[0xF as usize] = if VY > VX { 1 } else { 0 };

                        self.V[X as usize] -= VY as u8;
                        self.pc += 2;
                    }

                    0x0006 => {
                        p!(:"Opcode 8XY6: Shifts VX right by one (div by 2)");
                        // Opcode 8XY6: Shifts VX right by one (div by 2).
                        // If the least-significant bit of VX is 1, then VF is set to 1, otherwise 0.
                        // let X = (self.opcode & 0x0F00) >> 8;
                        // let VX = self.V[X as usize];
                        let (X, VX) = self.vx();
                        // Save LSB in VF
                        self.V[0xF as usize] = VX & 0x1;
                        self.V[X as usize] >>= 1;
                        self.pc += 2;
                    }

                    0x0007 => {
                        p!(:"Opcode 8XY7: Sets VX to (VY-VX)");
                        // Opcode 8XY7: Sets VX to (VY-VX)

                        // So now instead of doing THIS:
                        // let X = (self.opcode & 0x0F00) >> 8;
                        // let VX = self.V[X as usize] as u16;
                        // let Y = (self.opcode & 0x00F0) >> 4;
                        // let VY = self.V[Y as usize] as u16;

                        // I just do this:
                        let ((X, VX), (_, VY)) = self.vx_vy();
                        // Set the borrow flag
                        self.V[0xF as usize] = if VY > VX { 1 } else { 0 };

                        self.V[X as usize] = VY - VX;
                        self.pc += 2;
                    }
                    0x000E => {
                        p!(:"Opcode 8XYE: Shifts VX left by one.");
                        // Opcode 8XYE: Shifts VX left by one.
                        // VX receives the value of the most significant bit before the shift.
                        let (X, VX) = self.vx();
                        self.V[0xF as usize] = VX & 0x80;
                        self.V[X as usize] <<= 1;
                    }

                    op @ _ => {
                        eprintln!("Unknown opcode [0x8000#04x{}]", op);
                    }
                }
            }

            0x9000 => {
                p!(:"Opcode 9XY0: Skips the next instruction if VX != VY.");
                // Opcode 9XY0: Skips the next instruction if VX != VY.
                let ((_, VX), (_, VY)) = self.vx_vy();
                if VX != VY {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            0xA000 => {
                p!(:"Opcode ANNN: Sets I to the address NNN");
                // Opcode ANNN: Sets I to the address NNN
                self.I = self.opcode & 0x0FFF;
                self.pc += 2;
            }

            0xB000 => {
                p!(:"Opcode BNNN: Jumps to the address NNN + V0");
                // Opcode BNNN: Jumps to the address NNN + V0
                self.pc = (self.opcode & 0x0FFF) + (self.V[0] as u16);
            }

            0xC000 => {
                p!(:"Opcode CXNN: Sets VX to (random_byte &  NN).");
                // Opcode CXNN: Sets VX to (random_byte &  NN).
                let mut rng = rand::thread_rng();
                let (X, _) = self.vx();
                let NN = (self.opcode & 0x00FF) as u8;
                self.V[X as usize] = rng.gen::<u8>() & NN;
                self.pc += 2;
            }

            0xD000 => {
                p!(:"Opcode DXYN: draw sprite at (VX, VY), w=8, h=N");
                /*  Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
                Each row of 8 pixels is read as bit-coded starting from memory location I.
                The I value doesn’t change after the execution of this instruction.
                As described above, VF is set to 1 if any screen pixels are flipped from set to unset when
                the sprite is drawn, and to 0 if that doesn’t happen. */
                self.draw_sprite();
                self.pc += 2;
            }

            // Testing opcodes starting with EX___
            0xE000 => {
                match self.opcode & 0x00FF {

                    0x009E => {
                        p!(:"Opcode EX9E: Skips the next instruction if the key");
                        // Opcode EX9E: Skips the next instruction if the key
                        // stored in VX is pressed
                        let (_, VX) = self.vx();
                        if self.keypad[VX as usize] != 0 {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }

                    0x00A1 => {
                        p!(:"Opcode EXA1: Skips the next instruction if the key stored in");
                        // Opcode EXA1: Skips the next instruction if the key stored in
                        // VX is not pressed.
                        let (_, VX) = self.vx();
                        if self.keypad[VX as usize] == 0 {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    op => {
                        eprintln!("Unknown opcode EX#04x{}", op);
                    }
                }
            }

            // Testing opcodes starting with FX___
            0xF000 => {
                match self.opcode & 0x00FF {

                    0x0007 => {
                        p!(:"Opcode FX07: Sets VX to the value of the delay timer");
                        // Opcode FX07: Sets VX to the value of the delay timer
                        let (X, _) = self.vx();
                        self.V[X as usize] = self.delay_timer;
                        self.pc += 2;
                    }

                    0x000A => {
                        p!(:"Opcode FX0A: Wait for a key press, store the value of the key in Vx.");
                        // Opcode FX0A: Wait for a key press, store the value of the key in Vx.
                        let mut key_was_pressed = false;
                        for i in 0..16 {
                            if self.keypad[i as usize] != 0 {
                                let (X, _) = self.vx();
                                self.V[X as usize] = i;
                                key_was_pressed = true;
                                // TODO: break here?
                            }
                        }

                        if key_was_pressed {
                            self.pc += 2;
                        } else {
                            // A key was not pressed, so we try this operation again
                            // TODO: make sure timers are not decreases when this case happens?
                        }
                    }

                    0x0015 => {
                        p!(:"Opcode FX15: Set the delay timer to VX");
                        // Opcode FX15: Set the delay timer to VX
                        let (_, VX) = self.vx();
                        self.delay_timer = VX;
                        self.pc += 2;
                    }

                    0x0018 => {
                        p!(:"Opcode FX18: Set the sound timer to VX");
                        // Opcode FX18: Set the sound timer to VX
                        let (_, VX) = self.vx();
                        self.sound_timer = VX;
                        self.pc += 2;
                    }

                    0x001E => {
                        p!(:"Opcode FX1E: Adds VX to I.");
                        // Opcode FX1E: Adds VX to I.
                        // If the sum causes overflow, VF is set to one.
                        // If not, VF is set to zero.
                        let (_, VX) = self.vx();
                        self.V[0xF as usize] = if self.I + (VX as u16) > 0xFFF 
                                               { 1 } else { 0 };
                        self.I  += VX as u16;
                        self.pc += 2;
                    }

                    0x0029 => {
                        p!(:"Opcode FX29: Sets I to the location of the sprite for the character in VX.");
                        // Opcode FX29: Sets I to the location of the sprite for the character
                        // in VX.
                        let (_, VX) = self.vx();
                        // TODO: Verify if the fonts must start getting loaded from 0x50.
                        self.I   = (VX as u16) * 0x5;
                        self.pc += 2; 
                    }

                    0x0033 => {
                        p!(:"Opcode FX33: Stores the BCD representation of VX in mem. at I, I+1 and I+2.");
                        // Opcode FX33: Stores the BCD representation of VX in memory locations
                        // I, I+1 and I+2.
                        // The hundreds digit will be stored at I
                        // The tens digit will be stored at I+1
                        // And the ones digit stored at I+2 
                        let I = self.I;
                        let (_, VX) = self.vx();
                        let mut value = VX;
                        // We'll place the values in reverse order
                        // Ones place
                        self.memory[(I+2) as usize] = value % 10;
                        value /= 10;

                        // Tens place
                        self.memory[(I+1) as usize] = value % 10;
                        value /= 10;

                        // Hundreds place
                        self.memory[I as usize] = value % 10;

                        self.pc += 2;
                    }

                    0x0055 => {
                        p!(:"Opcode FX55: Stores the value of V0..VX on the memory, starting at I.");
                        // Opcode FX55: Stores the value of all registers, V0, V1, ..., VX
                        // on the memory, starting at location I.
                        let (X, _) = self.vx();
                        let I = self.I as u8;
                        for i in 0..=X {
                            self.memory[(I+i) as usize] = self.V[i as usize];
                        }
                        // TODO (quirk?): do I += X+1 ?
                        self.I += (X + 1) as u16;
                        self.pc += 2;
                    }

                    0x0065 => {
                        p!(:"Opcode FX65: Reads V0..VX from memory, starting at I.");
                        // Opcode FX65: Sets V0, V1, ... Vx to the values in memory, starting
                        // at location I.
                        let (X, _) = self.vx();
                        let I = self.I as u8;
                        for i in 0..=X {
                            self.V[i as usize] = self.memory[(I + i) as usize];
                        }
                        // TODO: quirk -- do I += X+1
                        self.I += (X+1) as u16;
                        self.pc += 2;
                    }

                    op => {
                        eprintln!("Unknown opcode FX#04x{}", op);
                    }
                }
            }

            op @ _ => {
                eprintln!("Unknown opcode #08x{}", op);
            }
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // Buzz!
            }
            self.sound_timer -= 1;
        }
    }
}
