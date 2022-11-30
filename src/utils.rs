/// Pre-reading data from memory increases speed slightly for some operations
#[inline(always)]
pub(crate) fn foreach_with_pre_reading<D, I>(
    mut iter: impl Iterator<Item = I>,
    read_data: fn(src: I) -> D,
    process_data: fn(data: D),
) {
    let mut next_data: D;
    if let Some(src) = iter.next() {
        next_data = read_data(src);
        for src in iter {
            let data = next_data;
            next_data = read_data(src);
            process_data(data);
        }
        process_data(next_data);
    }
}
