#[inline(always)]
pub fn ifdiv(n: usize, d: usize) -> usize {
    n / d
}

#[inline(always)]
pub fn icdiv(n: usize, d: usize) -> usize {
    ifdiv(n + d - 1, d)
}

#[inline(always)]
pub fn queue_shift_u8(n: &mut u8) {
    *n = *n & 1 | *n << 1;
}

#[inline(always)]
pub fn queue_set_u8(n: &mut u8) {
    *n |= 1;
}

#[inline(always)]
pub fn queue_clear_u8(n: &mut u8) {
    *n &= !1;
}
