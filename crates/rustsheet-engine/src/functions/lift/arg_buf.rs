//! `ArgBuf`: the argument values of one call, inline for up to [`INLINE_ARGS`] arguments
//! (SPEC-0011 "The driver"), so an unlifted call with few arguments does not touch the heap to
//! hold its values. Longer calls fall back to a `Vec`.

/// Arguments held without a heap allocation.
pub(crate) const INLINE_ARGS: usize = 8;

/// A fixed-length buffer of `T`: an inline array of [`INLINE_ARGS`] slots, or a `Vec`.
pub(crate) enum ArgBuf<T> {
    /// `len` slots of `items` are in use; the rest hold the filler.
    Inline { len: usize, items: [T; INLINE_ARGS] },
    /// More than [`INLINE_ARGS`] slots.
    Heap(Vec<T>),
}

impl<T> ArgBuf<T> {
    /// `len` slots, each set to `fill()`.
    pub(crate) fn filled(len: usize, fill: impl Fn() -> T) -> Self {
        if len <= INLINE_ARGS {
            ArgBuf::Inline {
                len,
                items: std::array::from_fn(|_| fill()),
            }
        } else {
            ArgBuf::Heap((0..len).map(|_| fill()).collect())
        }
    }

    /// The slots in use.
    pub(crate) fn as_slice(&self) -> &[T] {
        match self {
            ArgBuf::Inline { len, items } => items.get(..*len).unwrap_or_default(),
            ArgBuf::Heap(items) => items,
        }
    }

    /// The slots in use, writable.
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            ArgBuf::Inline { len, items } => items.get_mut(..*len).unwrap_or_default(),
            ArgBuf::Heap(items) => items,
        }
    }

    /// Whether the slots live inline (no heap allocation).
    #[cfg(test)]
    pub(crate) fn is_inline(&self) -> bool {
        matches!(self, ArgBuf::Inline { .. })
    }
}

impl<T> IntoIterator for ArgBuf<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    /// The slots in use, by value (the lifted path only, where the values become a `Vec` anyway).
    fn into_iter(self) -> Self::IntoIter {
        match self {
            ArgBuf::Inline { len, items } => {
                let mut items = Vec::from(items);
                items.truncate(len);
                items.into_iter()
            }
            ArgBuf::Heap(items) => items.into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ArgBuf, INLINE_ARGS};

    #[test]
    fn short_buffers_are_inline_and_long_ones_on_the_heap() {
        let mut buf = ArgBuf::filled(3, || 0);
        assert!(buf.is_inline());
        buf.as_mut_slice()[1] = 7;
        assert_eq!(buf.as_slice(), &[0, 7, 0]);
        assert_eq!(buf.into_iter().collect::<Vec<_>>(), vec![0, 7, 0]);
        let long = ArgBuf::filled(INLINE_ARGS + 1, || 1);
        assert!(!long.is_inline());
        assert_eq!(long.as_slice().len(), INLINE_ARGS + 1);
        assert_eq!(long.into_iter().count(), INLINE_ARGS + 1);
    }
}
