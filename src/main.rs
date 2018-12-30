use std::env;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub enum BFIError {
    Io(std::io::Error),
    MissingClosingBrackets,
    MissingOpeningBrackets,
}

impl std::fmt::Display for BFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            BFIError::Io(ref err) => write!(f, "{}", err),
            BFIError::MissingClosingBrackets => write!(f, "Missing closing bracket(s)"),
            BFIError::MissingOpeningBrackets => write!(f, "Missing opening bracket(s)"),
        }
    }
}

impl From<std::io::Error> for BFIError {
    fn from(err: std::io::Error) -> BFIError {
        BFIError::Io(err)
    }
}

pub struct BFI {
    c: String,
}

impl BFI {
    pub fn new(s: String) -> Self {
        Self {
            c: s,
        }
    }

    pub fn from_file(file_path: String) -> Result<Self, BFIError> {
        let mut code = String::new();
        let mut file = File::open(file_path)?;
        file.read_to_string(&mut code)?;

        Ok(Self::new(code))
    }

    pub fn check_syntax(&self) -> Result<(), BFIError> {
        let mut ob = 0;
        let mut cb = 0;
        for c in self.c.chars() {
            match c {
                '[' => ob += 1,
                ']' => cb += 1,
                _ => (),
            };
        };

        if ob > cb {
            Err(BFIError::MissingClosingBrackets)
        } else if ob < cb {
            Err(BFIError::MissingOpeningBrackets)
        } else {
            Ok(())
        }
    }
}

fn main() -> Result<(), BFIError> {
    for argument in env::args().skip(1) {
        let bfi = BFI::from_file(argument)?;
        bfi.check_syntax()?;
        println!("{}", bfi.c);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::BFI;
    use crate::BFIError;

    #[test]
    fn test_check_syntax() {
        let bfi = BFI::new("[->+<]".to_string());
        assert!(bfi.check_syntax().is_ok());

        let bfi = BFI::new("+->[".to_string());
        assert!(if let BFIError::MissingClosingBrackets = bfi.check_syntax().unwrap_err() {
            true
        } else {
            false
        });

        let bfi = BFI::new("[]+-]".to_string());
        assert!(if let BFIError::MissingOpeningBrackets = bfi.check_syntax().unwrap_err() {
            true
        } else {
            false
        });
    }
}

