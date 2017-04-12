use std::mem;

/// Typetags in an OSC packet USUALLY start with a comma, but not always.
/// This Iterator adapts them to NEVER start with a comma.
pub struct MaybeSkipComma<I> {
    iter: I,
    not_first: bool,
}

impl<I> MaybeSkipComma<I> where I: Iterator<Item=u8> {
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter,
            not_first: false,
        }
    }
}
impl<I> Iterator for MaybeSkipComma<I> where I: Iterator<Item=u8> {
    type Item = u8;
    /// For the first item in the iterator: drop it if it's a comma.
    /// For all subsequent items, yield them unchanged.
    fn next(&mut self) -> Option<u8> {
        for v in self.iter.by_ref() {
            let not_first = mem::replace(&mut self.not_first, true);
            if not_first || v != b',' {
                return Some(v)
            }
        }
        None
    }
}
