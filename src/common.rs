#[inline(always)]
pub fn ifdiv(n: usize, d: usize) -> usize {
    n / d
}

#[inline(always)]
pub fn icdiv(n: usize, d: usize) -> usize {
    ifdiv(n + d - 1, d)
}
