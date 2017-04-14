use serde::de::{Deserializer, DeserializeSeed, SeqVisitor};

use super::error::{Error, ResultE};

/// If we want to deserialize an entire sequence simultaneously,
/// e.g. reading (u32, u32) from the bitstream atomically,
/// then we need to call visitor.visit_seq(<something>).
/// IterVisitor serves the role of <something>, providing an adapter
/// that will visit each element in an iterator.
///
/// 
/// # Examples
/// visitor.visit_seq(IterVisitor([1u32, 2u32].into_iter()))
/// 
pub struct IterVisitor<I>(pub I);


impl<I> SeqVisitor for IterVisitor<I>
    where I: Iterator, I::Item : Deserializer<Error=Error>
{
    type Error = Error;
    fn visit_seed<T>(&mut self, seed: T) -> ResultE<Option<T::Value>>
        where T: DeserializeSeed
    {
        match self.0.next() {
            // End of sequence
            None => Ok(None),
            // Serialize the current item
            Some(value) => seed.deserialize(value).map(Some)
        }
    }
}

