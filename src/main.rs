#[macro_use] extern crate p_macro;
mod chip8;
mod cli;
mod rom;
use sdl2::{self, pixels::{Color, PixelFormatEnum}, event::Event, keyboard::Keycode};
use sdl2::rect::Rect;


// The CHIP-8 has a 64x32 screen
const SCREEN_SIZE: (u32, u32) = (64, 32);
// const BLACK: Color = Color::RGB(0, 0, 0);

macro_rules! catch {
    ($a:expr) => {
        if let Err(err) = $a {
            eprintln!("Error: {}", err);
            return;
        }
    };
}

fn main() {
	let cfg = cli::Config::new();
    let mut vm = chip8::VirtualMachine::new();
    catch!(cfg);
    let cfg = cfg.unwrap();
    println!("{:?}", cfg);
    let cart = rom::Cartridge::new(cfg.filename.clone());
    catch!(cart);
    let cart = cart.unwrap();
    vm.load_rom(cart);
    println!("{}", cart.size);

    let sdl_context = sdl2::init();
    catch!(sdl_context);
    let sdl_context = sdl_context.unwrap();
    let video_subsystem = sdl_context.video();
    catch!(video_subsystem);
    let video_subsystem = video_subsystem.unwrap();

    // Window title shows the loaded ROM
    let title = format!("lascaoito [{}]", cfg.filename.to_string());

    // Window dimensions
    let width  = SCREEN_SIZE.0 * (cfg.scale as u32);
    let height = SCREEN_SIZE.1 * (cfg.scale as u32);
    
    let window = video_subsystem.window(&title, width, height)
        .position_centered()
        .build();
    if let Err(err) = window {
        eprintln!("Error: {}", err);
        return;
    }
    let window = window.unwrap();
    drop(title);
    let canvas = window.into_canvas().build();
    
    catch!(canvas);
    let mut canvas = canvas.unwrap();
    // canvas.set_draw_color(BLACK);

    let texture_creator = canvas.texture_creator();

    // let surface = Surface::new()

    // let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, 64, 32);

    // let texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGBA8888,  64, 32);
    // catch!(texture);
    // let mut texture = texture.unwrap();

    canvas.clear();
    canvas.present();



    let event_pump = sdl_context.event_pump();
    catch!(event_pump);
    let mut event_pump = event_pump.unwrap();



    'main_loop: loop {
        for event in event_pump.poll_iter() 
        {
            match event 
            {
                Event::Quit { .. } | 
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main_loop;
                }
                _ => {}
            }
        }
        vm.run_cycle();
        if vm.draw_to_screen {
            canvas.clear();
            for (y, row) in vm.graphics.iter().enumerate() {
                for (x, &pixcol) in row.iter().enumerate() {
                    let x = (x as i32) * (cfg.scale as i32);
                    let y = (y as i32) * (cfg.scale as i32);

                    let color = if pixcol == 0 {
                        Color::RGB(0, 0, 0)
                    } else {
                        Color::RGB(0, 250, 0)
                    };

                    let scale = cfg.scale as u32;

                    let fill_result = canvas.fill_rect(
                        Rect::new(x, y, scale, scale)
                    );

                    catch!(fill_result);
                }
            }
            // canvas.present();
            vm.draw_to_screen = false;
        }
        canvas.present();
    };
}
