const WORDS: usize = 16;

pub(crate) const SLOTS: u32 = (WORDS * 64) as u32;

pub(crate) const EMPTY: Mask = Mask([0; WORDS]);

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Mask([u64; WORDS]);

pub(crate) fn bit(slot: u32) -> Mask {
    let mut mask = EMPTY;
    mask.0[(slot / 64) as usize] = 1 << (slot % 64);
    mask
}

pub(crate) fn contains(mask: &Mask, other: Mask) -> bool {
    mask.0
        .iter()
        .zip(other.0)
        .all(|(mine, wanted)| wanted & !mine == 0)
}

pub(crate) fn slots(mask: &Mask) -> impl Iterator<Item = u32> + '_ {
    mask.0.iter().enumerate().flat_map(|(word, bits)| {
        let mut left = *bits;
        std::iter::from_fn(move || {
            (left != 0).then(|| {
                let low = left.trailing_zeros();
                left &= left - 1;
                word as u32 * 64 + low
            })
        })
    })
}

pub(crate) fn without(mut mask: Mask, other: Mask) -> Mask {
    for (mine, taken) in mask.0.iter_mut().zip(other.0) {
        *mine &= !taken;
    }
    mask
}

pub(crate) fn with(mut mask: Mask, other: Mask) -> Mask {
    for (mine, added) in mask.0.iter_mut().zip(other.0) {
        *mine |= added;
    }
    mask
}
