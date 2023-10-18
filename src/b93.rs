use std::{
    ascii::escape_default,
    fmt,
    io::{self, BufRead, Read, Write},
    error,
};

use rand::Rng;

#[derive(Debug)]
pub enum Error {
    InvalidCharacter(i64),
    InvalidInstruction(u8),
    InvalidNumeric(String),
    IOError(io::Error),
    PlayfieldTooWide,
    PlayfieldTooTall,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidCharacter(n) =>
                write!(f, "attempt to output {} as character", n),
            Error::InvalidInstruction(i) =>
                write!(f, "invalid instruction: '{}'", escape_default(*i)),
            Error::InvalidNumeric(s) =>
                write!(f, "attempt to input '{}' as number", s),
            Error::IOError(e) =>
                write!(f, "IO error: '{}'", e),
            Error::PlayfieldTooWide =>
                write!(f, "playfield too wide"),
            Error::PlayfieldTooTall =>
                write!(f, "playfield too tall"),
        }
    }
}

impl error::Error for Error { }

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct B93 {
    playfield: [[u8; 80]; 25],
    stack: Vec<i64>,
    i: u8,
    j: u8,
    dir: Direction,
    bridge: bool,
    string: bool,
}

impl B93 {
    pub fn new(playfield: [[u8; 80]; 25]) -> Self {
        Self {
            playfield,
            stack: Vec::new(),
            i: 0,
            j: 0,
            dir: Direction::Right,
            bridge: false,
            string: false,
        }
    }

    pub fn from_stream<R: Read>(rdr: &mut R) -> Result<Self, Error> {
        let mut buf = Vec::new();
        rdr.read_to_end(&mut buf)?;

        let mut playfield = [[b' '; 80]; 25];
        let mut i = 0;
        let mut j = 0;
        let mut maybe_crlf = false;
        for b in buf {
            if maybe_crlf && b == b'\n' {
                maybe_crlf = false;
                continue;
            }

            if b == b'\r' {
                maybe_crlf = true;
            }

            if b == b'\r' || b == b'\n' {
                i += 1;
                j = 0;
                continue;
            }

            if i >= 25 {
                return Err(Error::PlayfieldTooWide);
            }

            if j >= 80 {
                return Err(Error::PlayfieldTooTall);
            }

            playfield[i][j] = b;
            j += 1;
        }

        Ok(Self::new(playfield))
    }

    pub fn next_instruction(&self) -> u8 {
        self.playfield[self.i as usize][self.j as usize]
    }

    pub fn step<R, W, Rand>(&mut self, rdr: &mut R, wtr: &mut W, rng: &mut Rand
      ) -> Result<Option<()>, Error>
      where R: BufRead, W: Write, Rand: Rng {
        if self.bridge {
            self.bridge = false;
            self.advance_pc();
            return Ok(Some(()));
        }

        if self.string {
            match self.next_instruction() {
                b'"' => self.string = false,
                c => self.push(c as i64),
            };
            self.advance_pc();
            return Ok(Some(()));
        }

        match self.next_instruction() {
            b' ' => { },
            b'@' => return Ok(None),
            b'^' => self.dir = Direction::Up,
            b'v' => self.dir = Direction::Down,
            b'<' => self.dir = Direction::Left,
            b'>' => self.dir = Direction::Right,
            b'?' => {
                self.dir = match rng.gen_range(0..4) {
                    0 => Direction::Up,
                    1 => Direction::Down,
                    2 => Direction::Left,
                    3 => Direction::Right,
                    _ => panic!("impossible RNG result"),
                }
            },
            b'"' => self.string = true,
            b'+' => {
                let x = self.pop();
                let y = self.pop();
                self.push(x + y);
            },
            b'-' => {
                let x = self.pop();
                let y = self.pop();
                self.push(x - y);
            },
            b'*' => {
                let x = self.pop();
                let y = self.pop();
                self.push(x * y);
            },
            b'/' => {
                let x = self.pop();
                let y = self.pop();
                self.push(x / y);
            },
            b'%' => {
                let x = self.pop();
                let y = self.pop();
                self.push(x % y);
            },
            b'!' => {
                if self.pop() != 0 {
                    self.push(0)
                } else {
                    self.push(1)
                }
            },
            b'_' => {
                if self.pop() != 0 {
                    self.dir = Direction::Left;
                } else {
                    self.dir = Direction::Right;
                }
            },
            b'|' => {
                if self.pop() != 0 {
                    self.dir = Direction::Up;
                } else {
                    self.dir = Direction::Down;
                }
            },
            b'&' => {
                // spec unclear; I'm saying this must be line-buffered
                let mut buf = String::new();
                rdr.read_line(&mut buf)?;
                if let Ok(val) = buf.trim().parse() {
                    self.push(val);
                } else {
                    return Err(Error::InvalidNumeric(buf));
                }
            },
            b'~' => {
                let mut buf = [0u8];
                rdr.read_exact(&mut buf)?;
                self.push(buf[0] as i64);
            },
            b'.' => write!(wtr, "{} ", self.pop())?,
            b',' => {
                let val = self.pop();
                if val < 0 || val > 127 {
                    return Err(Error::InvalidCharacter(val));
                }
                // safety: val is a valid u8 due to range check above
                // (also a valid ASCII char)
                write!(wtr, "{}",
                  TryInto::<u8>::try_into(val).unwrap() as char)?
            },
            b'#' => self.bridge = true,
            b':' => self.push(self.peek()),
            b'$' => { self.pop(); },
            b'\\' => {
                let x = self.pop();
                let y = self.pop();
                self.push(x);
                self.push(y);
            },
            b'`' => {
                let x = self.pop();
                let y = self.pop();
                if x > y {
                    self.push(1);
                } else {
                    // also unclear from spec!
                    self.push(0);
                }
            },
            b'g' => {
                let y = self.pop();
                let x = self.pop();
                if y < 0 || y >= 80 || x < 0 || x >= 25 {
                    self.push(b' ' as i64);
                } else {
                    self.push(self.playfield[x as usize][y as usize] as i64);
                }
            },
            b'p' => {
                let y = self.pop();
                let x = self.pop();
                let val = self.pop();
                if y >= 0 && y < 80 && x >= 0 && x < 25 {
                    // unclear what to do if i64 out of bounds for u8?
                    self.playfield[x as usize][y as usize] = val as u8;
                }
            },
            b => return Err(Error::InvalidInstruction(b)),
        };
        self.advance_pc();
        Ok(Some(()))
    }

    fn advance_pc(&mut self) {
        match self.dir {
            Direction::Up =>
                self.i = if self.i == 0 { 24 } else { self.i - 1 },
            Direction::Down =>
                self.i = (self.i + 1) % 25,
            Direction::Left =>
                self.j = if self.j == 0 { 79 } else { self.j - 1 },
            Direction::Right =>
                self.j = (self.j + 1) % 80,
        };
    }

    fn push(&mut self, val: i64) {
        self.stack.push(val)
    }

    fn pop(&mut self) -> i64 {
        self.stack.pop().unwrap_or(0)
    }

    // the spec is real unclear on this, or what dup should do on empty
    fn peek(&self) -> i64 {
        self.stack.first().copied().unwrap_or(0)
    }
}

impl Default for B93 {
    fn default() -> Self {
        Self::new([[b' '; 80]; 25])
    }
}
