use std::{
    env::args,
    error::Error,
    io,
    fs::File,
};

use rand::thread_rng;

pub mod b93;
use b93::B93;

fn main() -> Result<(), Box<dyn Error>> {
    let argv: Vec<_> = args().collect();
    let mut b93 = match &argv[..] {
        [_, file] => B93::from_stream(&mut File::open(file)?)?,
        [_] => B93::from_stream(&mut io::stdin())?,
        _ => Err("too many source files provided")?,
    };
    let mut inp = io::stdin().lock();
    let mut out = io::stdout();
    let mut rng = thread_rng();
    while let Some(()) = b93.step(&mut inp, &mut out, &mut rng)? { }
    Ok(())
}
