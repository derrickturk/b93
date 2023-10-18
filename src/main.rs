/*
Copyright © 2023 Derrick W. Turk

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the “Software”), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or
substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

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
