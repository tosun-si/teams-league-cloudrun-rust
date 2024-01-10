// Itertools provides a map_ok extension to iterators that maps Result::Ok values to
// plain values, which is inconvenient when the mapping functions also returns a Result as
// it leads to nested results.
// (see https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.map_ok)
//
// This try_map extension maps Result::Ok values to another Result, allowing to chain faillible
// mapping functions. The error from the second result has to be buildable from the first result's
// error using From::from

#[derive(Clone)]
pub struct TryMapIterator<I, F> {
    iter: I,
    f: F,
}

impl<I, F, A, B, EA, EB> Iterator for TryMapIterator<I, F>
    where
        I: Iterator<Item = Result<A, EA>>,
        EB: From<EA>,
        F: FnMut(A) -> Result<B, EB>,
{
    type Item = Result<B, EB>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            // Map the value
            Some(Ok(value)) => Some((self.f)(value)),
            // Convert the error
            Some(Err(err)) => Some(Err(EB::from(err))),
        }
    }
}

pub trait TryMap {
    fn try_map<F, A, B, EA, EB>(self, func: F) -> TryMapIterator<Self, F>
        where
            Self: Sized + Iterator<Item = Result<A, EA>>,
            F: FnMut(A) -> Result<B, EB>,
            EB: From<EA>,
    {
        TryMapIterator {
            iter: self,
            f: func,
        }
    }
}

// Blanket implementation on iterators that return a Result
impl<I, T, E> TryMap for I
    where
        I: Sized + Iterator<Item = Result<T, E>>,
{
}

#[cfg(test)]
mod tests {
    use super::TryMap;

    #[test]
    fn test_try_map() {
        let x: Vec<isize> = vec![1, 2, 3];

        let y = x.iter()
            .map(|x| if *x > 2 { Err("Too big") } else { Ok(*x) } )
            .try_map(|v| if v%2 == 0 { Ok(v) } else { Err("Not even") })
            .collect::<Vec<_>>();

        assert_eq!(y, vec![Err("Not even"), Ok(2), Err("Too big")]);
    }
}
