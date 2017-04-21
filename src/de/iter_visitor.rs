use serde::de::{Deserializer, DeserializeSeed, SeqAccess};

use error::{Error, ResultE};

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


impl<'de, I> SeqAccess<'de> for IterVisitor<I>
    where I: Iterator, I::Item : Deserializer<'de, Error=Error>
{
    type Error = Error;
    fn next_element_seed<T>(&mut self, seed: T) -> ResultE<Option<T::Value>>
        where T: DeserializeSeed<'de>
    {
        match self.0.next() {
            // End of sequence
            None => Ok(None),
            // Serialize the current item
            Some(value) => seed.deserialize(value).map(Some)
        }
    }
}

