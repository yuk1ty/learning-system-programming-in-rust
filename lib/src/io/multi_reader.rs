use std::{collections::VecDeque, io::Read};

/// Takes multiple `std::io::Read` at once.
/// This is inspired by `io.MultiReader` in Go.
///
/// # Example
///
/// ```
/// use std::{
///     io::{copy, stdout, Read},
///     usize,
/// };
/// use lib::io::MultiReader;
///
/// fn main() -> std::io::Result<()> {
///     let header = "---- HEADER ----\n".as_bytes();
///     let content = "Example of MultiReader\n".as_bytes();
///     let footer = "---- FOOTER ----\n".as_bytes();
///     let mut multi_reader = MultiReader::new(vec![header, content, footer]);
///     copy(&mut multi_reader, &mut stdout())?;
///     Ok(())
/// }
/// ```
pub struct MultiReader<R: Read> {
    readers: VecDeque<R>,
    /// Points to where we read right now.
    current: Option<R>,
}

impl<R: Read> MultiReader<R> {
    /// Creates `MultiReader`. `pos` is set to 0 by default.
    pub fn new(mut readers: VecDeque<R>) -> Self {
        let current = readers.pop_front();
        Self { readers, current }
    }
}

impl<R: Read> Read for MultiReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match self.current.take() {
                Some(ref mut r) => {
                    let n = r.read(buf)?;
                    if n > 0 {
                        return Ok(n);
                    }
                }
                None => return Ok(0),
            }
            self.current = self.readers.pop_front();
        }
    }
}
