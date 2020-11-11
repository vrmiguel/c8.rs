mod chip8;
mod cli;
mod sdl2;



struct Graphics {
    config: cli::Config,
    vm    : chip8::VirtualMachine,
}

impl Graphics {
    fn new() -> Graphics {
        
    }
}