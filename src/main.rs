use std::env;
use std::fs::File;
use std::i8;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub enum BFIError {
    Io(std::io::Error),
    MissingClosingBrackets,
    MissingOpeningBrackets,
    OutOfMemory,
    ArithmeticOverflow,
}

impl std::fmt::Display for BFIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            BFIError::Io(ref err) => write!(f, "{}", err),
            BFIError::MissingClosingBrackets => write!(f, "Missing closing bracket(s)"),
            BFIError::MissingOpeningBrackets => write!(f, "Missing opening bracket(s)"),
            BFIError::OutOfMemory => write!(f, "Pointer moved to out of range of memory"),
            BFIError::ArithmeticOverflow => write!(f, "Byte overflow"),
        }
    }
}

impl From<std::io::Error> for BFIError {
    fn from(err: std::io::Error) -> BFIError {
        BFIError::Io(err)
    }
}

#[derive(Debug)]
pub struct BFI {
    x: Vec<i8>,
    c: String,
    p: usize,
    pc: usize,
}

impl BFI {
    pub fn new(s: String) -> Self {
        Self {
            x: vec![0; 32767 + 1],
            c: s,
            p: 0,
            pc: 0,
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

    fn increment_pointer(&mut self) -> Result<(), BFIError> {
        if self.pc + 1 >= self.x.len() {
            Err(BFIError::OutOfMemory)
        } else {
            self.pc += 1;
            Ok(())
        }
    }

    fn decrement_pointer(&mut self) -> Result<(), BFIError> {
        if self.pc <= 0 {
            Err(BFIError::OutOfMemory)
        } else {
            self.pc -= 1;
            Ok(())
        }
    }

    fn increment_byte_at_pointer(&mut self) -> Result<(), BFIError> {
        if self.x[self.p] == i8::MAX {
            Err(BFIError::ArithmeticOverflow)
        } else {
            self.x[self.p] += 1;
            Ok(())
        }
    }

    fn decrement_byte_at_pointer(&mut self) -> Result<(), BFIError> {
        if self.x[self.p] == i8::MIN {
            Err(BFIError::ArithmeticOverflow)
        } else {
            self.x[self.p] -= 1;
            Ok(())
        }
    }

    fn output(&self, writer: &mut Write) -> Result<(), BFIError> {
        let buf = [self.x[self.p] as u8; 1];
        writer.write(&buf)?;
        Ok(())
    }

    pub fn interpret(&mut self, _reader: &Read, writer: &mut Write) -> Result<(), BFIError> {
        let chars_length = self.c.len();

        self.pc = 0;
        while self.pc < chars_length {
            let c = self.c.get(self.pc..=self.pc);

            match c {
                Some(">") => self.increment_pointer()?,
                Some("<") => self.decrement_pointer()?,
                Some("+") => self.increment_byte_at_pointer()?,
                Some("-") => self.decrement_byte_at_pointer()?,
                Some(".") => self.output(writer)?,
                Some(",") => (),
                Some("[") => (),
                Some("]") => (),
                _ => (),
            };

            self.pc += 1;
        }
        Ok(())
    }
}

fn main() -> Result<(), BFIError> {
    for argument in env::args().skip(1) {
        let mut bfi = BFI::from_file(argument)?;
        bfi.check_syntax()?;
        bfi.interpret(&std::io::stdin(), &mut std::io::stdout())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::i8;
    use std::io::Cursor;

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

    #[test]
    fn test_check_increment_pointer() {
        let mut bfi = BFI::new(".".to_string());
        assert_eq!(bfi.pc, 0);
        bfi.increment_pointer().unwrap();
        assert_eq!(bfi.pc, 1);
        bfi.increment_pointer().unwrap();
        assert_eq!(bfi.pc, 2);

        bfi.pc = bfi.x.len() - 2;
        bfi.increment_pointer().unwrap();
        assert_eq!(bfi.pc, bfi.x.len() - 1);
        assert!(if let BFIError::OutOfMemory = bfi.increment_pointer().unwrap_err() {
            true
        } else {
            false
        });
        assert_eq!(bfi.pc, bfi.x.len() - 1);
    }

    #[test]
    fn test_check_decrement_pointer() {
        let mut bfi = BFI::new(".".to_string());
        assert_eq!(bfi.pc, 0);
        assert!(if let BFIError::OutOfMemory = bfi.decrement_pointer().unwrap_err() {
            true
        } else {
            false
        });
        assert_eq!(bfi.pc, 0);

        bfi.pc = 1;
        bfi.decrement_pointer().unwrap();
        assert_eq!(bfi.pc, 0);

        bfi.pc = bfi.x.len() - 1;
        bfi.decrement_pointer().unwrap();
        assert_eq!(bfi.pc, bfi.x.len() - 2);
    }

    #[test]
    fn test_increment_byte_at_pointer() {
        let mut bfi = BFI::new(".".to_string());
        bfi.p = 0;
        assert_eq!(bfi.x[0], 0);
        bfi.increment_byte_at_pointer().unwrap();
        assert_eq!(bfi.x[0], 1);
        bfi.increment_byte_at_pointer().unwrap();
        assert_eq!(bfi.x[0], 2);

        bfi.p = 1;
        bfi.x[1] = i8::MAX - 1;
        bfi.increment_byte_at_pointer().unwrap();
        assert_eq!(bfi.x[1], i8::MAX);
        assert!(if let BFIError::ArithmeticOverflow = bfi.increment_byte_at_pointer().unwrap_err() {
            true
        } else {
            false
        });
        assert_eq!(bfi.x[1], i8::MAX);

        bfi.p = 2;
        bfi.x[2] = i8::MIN;
        bfi.increment_byte_at_pointer().unwrap();
        assert_eq!(bfi.x[2], i8::MIN + 1);
    }

    #[test]
    fn test_decrement_byte_at_pointer() {
        let mut bfi = BFI::new(".".to_string());
        bfi.p = 0;
        assert_eq!(bfi.x[0], 0);
        bfi.decrement_byte_at_pointer().unwrap();
        assert_eq!(bfi.x[0], -1);
        bfi.decrement_byte_at_pointer().unwrap();
        assert_eq!(bfi.x[0], -2);

        bfi.p = 1;
        bfi.x[1] = i8::MIN + 1;
        bfi.decrement_byte_at_pointer().unwrap();
        assert_eq!(bfi.x[1], i8::MIN);
        assert!(if let BFIError::ArithmeticOverflow = bfi.decrement_byte_at_pointer().unwrap_err() {
            true
        } else {
            false
        });
        assert_eq!(bfi.x[1], i8::MIN);

        bfi.p = 2;
        bfi.x[2] = i8::MAX;
        bfi.decrement_byte_at_pointer().unwrap();
        assert_eq!(bfi.x[2], i8::MAX - 1);
    }

    #[test]
    fn test_output() {
        let mut bfi = BFI::new(".".to_string());
        bfi.p = 0;
        bfi.x[0] = 0;

        let mut cursor = Cursor::new(Vec::new());
        bfi.output(&mut cursor).unwrap();
        assert_eq!(cursor.get_ref()[0], 0);

        bfi.p = 1;
        bfi.x[1] = 1;
        bfi.output(&mut cursor).unwrap();
        assert_eq!(cursor.get_ref()[0..2], [0, 1]);

        bfi.p = 2;
        bfi.x[2] = i8::MAX;
        bfi.output(&mut cursor).unwrap();
        assert_eq!(cursor.get_ref()[0..3], [0, 1, i8::MAX as u8]);

        bfi.p = 3;
        bfi.x[3] = i8::MIN;
        bfi.output(&mut cursor).unwrap();
        assert_eq!(cursor.get_ref()[0..4], [0, 1, i8::MAX as u8, i8::MIN as u8]);
    }
}

