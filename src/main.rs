#[macro_use] extern crate p_macro;
mod chip8;
mod cli;
mod rom;

fn main() {
	let cfg = cli::Config::new();
    let mut vm = chip8::VirtualMachine::new();
    if let Err(err) = cfg {
        eprintln!("Error: {}",  err);
        return;
    }
    let cfg = cfg.unwrap();
    let cart = rom::Cartridge::new(cfg.filename.clone());
    if let Err(err) = cart {
        eprintln!("Error: {}", err);
        return;
    }
    println!("{:?}", cfg);
    let cart = cart.unwrap();
    vm.load_rom(cart);
    println!("{}", cart.size);

    loop {
    	vm.run_cycle();
    	println!("{}", vm);
    }
}
