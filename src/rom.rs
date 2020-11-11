use std::fs::File;
use std::io::{Error, ErrorKind, Read};

/// A ROM may contain at max 4096-512 bytes, since 4096 bytes is the
/// maximum available amount of memory, and the first 512 bytes are  
/// reserved by the machine-specific interpreters.
const MAX_ROM_SIZE: u16 = 4096-512;

#[derive(Debug, Clone, Copy)]
pub struct Cartridge {
    // The data in the ROM
    pub data: [u8; MAX_ROM_SIZE as usize],
    // How many bytes are in the ROM
    pub size: u16
}

impl Cartridge {
    pub fn new(filename: String) ->  Result<Cartridge, Error>
    {
        let mut file = File::open(filename).expect("File not found!");
        let mut buffer = [0_u8; MAX_ROM_SIZE as usize];

        let file_size = file.metadata().unwrap().len();
        if file_size > (MAX_ROM_SIZE as u64)  {
            return Err(Error::new(ErrorKind::Other, "The supplied ROM is too big."));
        }
    
        let rom_size = if let Ok(bytes_read) = file.read(&mut buffer) {
            bytes_read
        } else {
            return Err(Error::new(ErrorKind::Other, "There's been a problem reading the ROM."));
        };
    
        Ok(Cartridge {
            data: buffer,
            size: rom_size as u16
        })
    }
}