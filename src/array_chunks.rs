use core::array::IntoIter;
use core::iter::Take;
use core::iter::{FusedIterator, Iterator};
use std::mem::MaybeUninit;

/// An iterator over `N` elements of the iterator at a time.
///
/// The chunks do not overlap. If `N` does not divide the length of the
/// iterator, then the last up to `N-1` elements will be omitted.
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ArrayChunks<I: Iterator, const N: usize> {
    iter: I,
    remainder: Option<Take<IntoIter<I::Item, N>>>,
}

impl<I, const N: usize> ArrayChunks<I, N>
where
    I: Iterator,
{
    pub fn new(iter: I) -> Self {
        assert_ne!(N, 0, "chunk size must be non-zero");
        Self {
            iter,
            remainder: None,
        }
    }

    /// Returns an iterator over the remaining elements of the original iterator
    /// that are not going to be returned by this iterator. The returned
    /// iterator will yield at most `N-1` elements.
    #[inline]
    pub fn into_remainder(self) -> Option<Take<IntoIter<I::Item, N>>> {
        self.remainder
    }
}

impl<I, const N: usize> Iterator for ArrayChunks<I, N>
where
    I: Iterator,
{
    type Item = [I::Item; N];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match next_chunk(&mut self.iter) {
            Ok(chunk) => Some(chunk),
            Err(remainder) => {
                // Make sure to not override `self.remainder` with an empty array
                // when `next` is called after `ArrayChunks` exhaustion.
                self.remainder.get_or_insert(remainder);
                None
            }
        }
        // self.try_for_each(ControlFlow::Break).break_value()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        (lower / N, upper.map(|n| n / N))
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count() / N
    }
}

#[inline]
fn next_chunk<I: Iterator + Sized, const N: usize>(
    iter: &mut I,
) -> Result<[I::Item; N], Take<IntoIter<I::Item, N>>> {
    iter_next_chunk(iter)
}

impl<I, const N: usize> FusedIterator for ArrayChunks<I, N> where I: FusedIterator {}

impl<I, const N: usize> ExactSizeIterator for ArrayChunks<I, N>
where
    I: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        self.iter.len() / N
    }
}

/// Pulls `N` items from `iter` and returns them as an array. If the iterator
/// yields fewer than `N` items, `Err` is returned containing an iterator over
/// the already yielded items.
///
/// Since the iterator is passed as a mutable reference and this function calls
/// `next` at most `N` times, the iterator can still be used afterwards to
/// retrieve the remaining items.
///
/// If `iter.next()` panicks, all items already yielded by the iterator are
/// dropped.
///
/// Used for [`Iterator::next_chunk`].
#[inline]
fn iter_next_chunk<T, const N: usize>(
    iter: &mut impl Iterator<Item = T>,
) -> Result<[T; N], Take<IntoIter<T, N>>> {
    let mut array = uninit_array::<T, N>();
    let r = iter_next_chunk_erased(&mut array, iter);
    match r {
        Ok(()) => {
            // SAFETY: All elements of `array` were populated.
            Ok(unsafe { array_assume_init(array) })
        }
        Err(initialized) => {
            // SAFETY: Only the first `initialized` elements were populated
            let array = unsafe { array_assume_init(array) };
            Err(array.into_iter().take(initialized))
            // Err(unsafe { IntoIter::new_unchecked(array, 0..initialized) })
        }
    }
}

#[inline(always)]
const fn uninit_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
    // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
    unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
}

#[inline(always)]
unsafe fn array_assume_init<T, const N: usize>(array: [MaybeUninit<T>; N]) -> [T; N] {
    // SAFETY:
    // * The caller guarantees that all elements of the array are initialized
    // * `MaybeUninit<T>` and T are guaranteed to have the same layout
    // * `MaybeUninit` does not drop, so there are no double-frees
    // And thus the conversion is safe
    let ret = unsafe {
        // core::intrinsics::assert_inhabited::<[T; N]>();
        (&array as *const _ as *const [T; N]).read()
    };

    // FIXME: required to avoid `~const Destruct` bound
    core::mem::forget(array);
    ret
}

/// Version of [`iter_next_chunk`] using a passed-in slice in order to avoid
/// needing to monomorphize for every array length.
///
/// Unfortunately this loop has two exit conditions, the buffer filling up
/// or the iterator running out of items, making it tend to optimize poorly.
#[inline]
fn iter_next_chunk_erased<T>(
    buffer: &mut [MaybeUninit<T>],
    iter: &mut impl Iterator<Item = T>,
) -> Result<(), usize> {
    let mut guard = Guard {
        array_mut: buffer,
        initialized: 0,
    };
    while guard.initialized < guard.array_mut.len() {
        let Some(item) = iter.next() else {
            // Unlike `try_from_fn_erased`, we want to keep the partial results,
            // so we need to defuse the guard instead of using `?`.
            let initialized = guard.initialized;
            core::mem::forget(guard);
            return Err(initialized);
        };

        // SAFETY: The loop condition ensures we have space to push the item
        unsafe { guard.push_unchecked(item) };
    }

    core::mem::forget(guard);
    Ok(())
}

/// Panic guard for incremental initialization of arrays.
///
/// Disarm the guard with `mem::forget` once the array has been initialized.
///
/// # Safety
///
/// All write accesses to this structure are unsafe and must maintain a correct
/// count of `initialized` elements.
///
/// To minimize indirection fields are still pub but callers should at least use
/// `push_unchecked` to signal that something unsafe is going on.
struct Guard<'a, T> {
    /// The array to be initialized.
    pub array_mut: &'a mut [MaybeUninit<T>],
    /// The number of items that have been initialized so far.
    pub initialized: usize,
}

impl<T> Guard<'_, T> {
    /// Adds an item to the array and updates the initialized item counter.
    ///
    /// # Safety
    ///
    /// No more than N elements must be initialized.
    #[inline]
    pub unsafe fn push_unchecked(&mut self, item: T) {
        // SAFETY: If `initialized` was correct before and the caller does not
        // invoke this method more than N times then writes will be in-bounds
        // and slots will not be initialized more than once.
        unsafe {
            self.array_mut
                .get_unchecked_mut(self.initialized)
                .write(item);
            self.initialized += 1;
        }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        debug_assert!(self.initialized <= self.array_mut.len());

        // SAFETY: this slice will contain only initialized objects.
        unsafe {
            core::ptr::drop_in_place(slice_assume_init_mut(
                self.array_mut.get_unchecked_mut(..self.initialized),
            ));
        }
    }
}

#[inline(always)]
unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY: similar to safety notes for `slice_get_ref`, but we have a
    // mutable reference which is also guaranteed to be valid for writes.
    unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut [T]) }
}
