use std::{
    error::Error,
    io,
};

use rand::thread_rng;

mod b93;
use b93::B93;

fn main() -> Result<(), Box<dyn Error>> {
    let mut b93 = B93::from_stream(&mut io::stdin())?;
    let mut inp = io::stdin().lock();
    let mut out = io::stdout();
    let mut rng = thread_rng();
    while let Some(()) = b93.step(&mut inp, &mut out, &mut rng)? { }
    Ok(())
}
