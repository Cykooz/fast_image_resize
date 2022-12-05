/// Pre-reading data from memory increases speed slightly for some operations
#[inline(always)]
pub(crate) fn foreach_with_pre_reading<D, I>(
    mut iter: impl Iterator<Item = I>,
    mut read_data: impl FnMut(I) -> D,
    mut process_data: impl FnMut(D),
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

macro_rules! test_log {
    ($s:expr) => {
        #[cfg(feature = "for_test")]
        {
            use crate::testing::log_message;
            log_message($s);
        }
    };
}
