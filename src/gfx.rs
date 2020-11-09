mod chip8;
mod cli;
mod sdl2;

// The CHIP-8 has a 64x32 screen
const SCREEN_SIZE: (i16, i16) = (64, 32);

struct Graphics {
    config: cli::Config,
    vm    : chip8::VirtualMachine,
}

impl Graphics {
    fn new() -> Graphics {
        
    }
}