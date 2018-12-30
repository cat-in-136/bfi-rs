use std::fs::File;
use std::io::Error;
use std::env;
use std::io::Read;

pub struct BFI {
    c: String,
}

impl BFI {
    pub fn new(s: String) -> Self {
        Self {
            c: s,
        }
    }

    pub fn from_file(file_path: String) -> Result<Self, Error> {
        let mut code = String::new();
        let mut file = File::open(file_path)?;
        file.read_to_string(&mut code)?;

        Ok(Self::new(code))
    }
}

fn main() -> Result<(), Error> {
    for argument in env::args().skip(1) {
        let bfi = BFI::from_file(argument)?;
        println!("{}", bfi.c);
    }
    Ok(())
}
