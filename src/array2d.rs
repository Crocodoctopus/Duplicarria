use std::ops::{Index, IndexMut, Range};

pub trait Index2d<T>: Index<T> {
    fn size(&self) -> (T, T);
    fn stride(&self) -> T;
}

pub trait Index2dMut<T>: Index2d<T> + IndexMut<T> {}

#[inline(always)]
pub fn for_each_sub_wrapping(
    w: usize,
    h: usize,
    xr: Range<usize>,
    yr: Range<usize>,
    mut f: impl FnMut(usize, usize, usize),
) {
    // If no iteration can occur, early exit.
    if xr.start >= xr.end || yr.start >= yr.end {
        return;
    }

    // How many times a wrap occurs, in both directions.
    let v_boundary_crosses = (xr.end - 1) / w - xr.start / w;
    //let v_boundary_crosses = (xr.end) / w - xr.start / w;
    let h_boundary_crosses = (yr.end - 1) / h - yr.start / h;

    // apply masks
    let x1 = xr.start % w;
    let x2 = (xr.end - 1) % w + 1;
    let y1 = yr.start % h;
    let y2 = (yr.end - 1) % h + 1;

    // hack
    let mut world_y = yr.start;
    #[rustfmt::skip] macro_rules! wrap_xloop {
        ($y:ident) => {{
            let index_base = ($y) * w;
            let mut world_x = xr.start;
            for x in x1..w { f(world_x, world_y, x + index_base); world_x+=1; }
            for _ in 1..v_boundary_crosses { for x in 0..w { f(world_x, world_y, x + index_base); world_x+=1; } }
            for x in 0..x2 { f(world_x, world_y, x + index_base); world_x+=1; }
        }}
    }
    #[rustfmt::skip] macro_rules! no_wrap_xloop {
        ($y:ident) => {{
            let index_base = ($y) * w;
            let mut world_x = xr.start;
            for x in x1..x2 { f(world_x, world_y, x + index_base); world_x+=1; }
        }};
    }
    #[rustfmt::skip] macro_rules! wrap_yloop {
        ($xloop:tt) => {{
            for y in y1..h { $xloop!(y); world_y += 1; }
            for _ in 1..h_boundary_crosses { for y in 0..h { $xloop!(y); world_y += 1; } }
            for y in 0..y2 { $xloop!(y); world_y += 1; }
        }}
    }
    #[rustfmt::skip] macro_rules! no_wrap_yloop {
        ($xloop:tt) => { for y in y1..y2 { $xloop!(y); world_y += 1; }}
    }

    match (h_boundary_crosses > 0, v_boundary_crosses > 0) {
        (true, true) => wrap_yloop!(wrap_xloop),
        (true, false) => wrap_yloop!(no_wrap_xloop),
        (false, true) => no_wrap_yloop!(wrap_xloop),
        (false, false) => no_wrap_yloop!(no_wrap_xloop),
    }
}

#[derive(Debug)]
pub enum Array2DErr {
    OutOfBounds,
}

#[derive(Debug)]
pub struct Array2D<T> {
    width: usize,
    height: usize,
    data: Box<[T]>,
}

impl<T> Index<usize> for Array2D<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for Array2D<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T> Index<usize> for &Array2D<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> Index<usize> for &mut Array2D<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for &mut Array2D<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T> Index2d<usize> for Array2D<T> {
    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn stride(&self) -> usize {
        self.width
    }
}

impl<T> Index2dMut<usize> for Array2D<T> {}

impl<T> Index2d<usize> for &Array2D<T> {
    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn stride(&self) -> usize {
        self.width
    }
}

impl<T> Index2d<usize> for &mut Array2D<T> {
    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn stride(&self) -> usize {
        self.width
    }
}

impl<T> Index2dMut<usize> for &mut Array2D<T> {}

impl<T: Copy> Array2D<T> {
    pub fn from_value(width: usize, height: usize, t: T) -> Self {
        let data = {
            let size = width * height;
            let mut vec = Vec::with_capacity(size);
            unsafe { vec.set_len(size) }
            for i in 0..size {
                std::mem::forget(std::mem::replace(&mut vec[i], t));
            }
            vec.into_boxed_slice()
        };

        Self {
            width,
            height,
            data,
        }
    }
}

impl<T> Array2D<T> {
    pub fn from_closure(width: usize, height: usize, f: impl Fn(usize, usize) -> T) -> Self {
        let data = {
            let size = width * height;
            let mut vec = Vec::with_capacity(size);
            unsafe { vec.set_len(size) }
            for y in 0..height {
                for x in 0..width {
                    std::mem::forget(std::mem::replace(&mut vec[x + y * width], f(x, y)));
                }
            }
            vec.into_boxed_slice()
        };

        Self {
            width,
            height,
            data,
        }
    }

    pub fn from_box(width: usize, height: usize, data: Box<[T]>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    pub fn into_raw(self) -> Box<[T]> {
        self.data
    }

    pub fn raw(&self) -> &Box<[T]> {
        &self.data
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x > self.width {
            return None;
        }
        self.data.get(x + y * self.width)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x >= self.width {
            return None;
        }
        self.data.get_mut(x + y * self.width)
    }

    pub fn get_wrapping(&self, x: usize, y: usize) -> &T {
        self.get(x % self.width, y % self.height).unwrap()
    }

    pub fn get_wrapping_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.get_mut(x % self.width, y % self.height).unwrap()
    }

    pub fn inbounds(&self, xr: Range<usize>, yr: Range<usize>) -> bool {
        let (width, height) = self.size();
        xr.start < width && yr.start < height && xr.end <= width && yr.end <= height
    }

    pub fn splice_wrapping(&mut self, xr: Range<usize>, yr: Range<usize>, data: Box<[T]>) {
        let mut i = data.into_vec().into_iter();
        self.for_each_sub_wrapping_mut(xr, yr, |_, _, t| {
            *t = i.next().unwrap();
        });
    }

    pub fn for_each(&self, mut f: impl FnMut(usize, usize, &T)) {
        let (w, h) = self.size();
        self.for_each_sub_wrapping(0..w, 0..h, |x, y, t| f(x, y, t));
    }

    pub fn for_each_mut(&mut self, mut f: impl FnMut(usize, usize, &mut T)) {
        let (w, h) = self.size();
        self.for_each_sub_wrapping_mut(0..w, 0..h, |x, y, t| f(x, y, t));
    }

    pub fn for_each_sub_wrapping(
        &self,
        xr: Range<usize>,
        yr: Range<usize>,
        mut f: impl FnMut(usize, usize, &T),
    ) {
        let (w, h) = self.size();
        for_each_sub_wrapping(w, h, xr, yr, |x, y, index| {
            f(x, y, &self.data[index]);
        });
    }

    pub fn for_each_sub_wrapping_mut(
        &mut self,
        xr: Range<usize>,
        yr: Range<usize>,
        mut f: impl FnMut(usize, usize, &mut T),
    ) {
        let (w, h) = self.size();
        for_each_sub_wrapping(w, h, xr, yr, |x, y, index| {
            f(x, y, &mut self.data[index]);
        });
    }
}

impl<T: Clone> Array2D<T> {
    pub fn clone_sub(&self, xr: Range<usize>, yr: Range<usize>) -> Result<Array2D<T>, Array2DErr> {
        if !self.inbounds(xr.clone(), yr.clone()) {
            return Err(Array2DErr::OutOfBounds);
        }

        return Ok(self.clone_sub_wrapping(xr, yr));
    }

    pub fn clone_sub_wrapping(&self, xr: Range<usize>, yr: Range<usize>) -> Array2D<T> {
        let width = xr.end - xr.start;
        let height = yr.end - yr.start;
        let mut arr = Vec::with_capacity(width * height);
        self.for_each_sub_wrapping(xr, yr, |_, _, t| arr.push(t.clone()));

        return Self {
            width,
            height,
            data: arr.into_boxed_slice(),
        };
    }
}

#[derive(Debug)]
pub struct FastArray2D<T> {
    w_exp: usize,
    x_mask: usize,
    y_mask: usize,
    data: Box<[T]>,
}

impl<T> Index<usize> for FastArray2D<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> Index2d<usize> for FastArray2D<T> {
    fn size(&self) -> (usize, usize) {
        self.size()
    }

    fn stride(&self) -> usize {
        self.size().0
    }
}

impl<T: Copy + std::fmt::Debug> FastArray2D<T> {
    pub fn from_value(w_exp: usize, h_exp: usize, t: T) -> Self {
        let width = 1 << w_exp;
        let height = 1 << h_exp;

        Self {
            w_exp,
            x_mask: width - 1,
            y_mask: height - 1,
            data: vec![t; width * height].into_boxed_slice(),
        }
    }

    pub fn from_closure(w_exp: usize, h_exp: usize, f: impl Fn(usize, usize) -> T) -> Self {
        let width = 1 << w_exp;
        let height = 1 << h_exp;

        let mut arr = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                arr.push(f(x, y));
            }
        }

        Self {
            w_exp,
            x_mask: width - 1,
            y_mask: height - 1,
            data: arr.into_boxed_slice(),
        }
    }

    pub fn from_box(w_exp: usize, h_exp: usize, data: Box<[T]>) -> Self {
        let width = 1 << w_exp;
        let height = 1 << h_exp;

        Self {
            w_exp,
            x_mask: width - 1,
            y_mask: height - 1,
            data,
        }
    }

    pub fn into_raw(self) -> Box<[T]> {
        self.data
    }

    pub fn size(&self) -> (usize, usize) {
        (self.x_mask + 1, self.y_mask + 1)
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.data.get(x + (y << self.w_exp))
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(x + (y << self.w_exp))
    }

    pub fn get_wrapping(&self, x: usize, y: usize) -> &T {
        self.get(x & self.x_mask, y & self.y_mask).unwrap()
    }

    pub fn get_wrapping_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.get_mut(x & self.x_mask, y & self.y_mask).unwrap()
    }

    pub fn inbounds(&self, xr: Range<usize>, yr: Range<usize>) -> bool {
        let (width, height) = self.size();
        xr.start < width && yr.start < height && xr.end <= width && yr.end <= height
    }

    pub fn clone_sub(&self, xr: Range<usize>, yr: Range<usize>) -> Result<Array2D<T>, Array2DErr> {
        if !self.inbounds(xr.clone(), yr.clone()) {
            return Err(Array2DErr::OutOfBounds);
        }

        return Ok(self.clone_sub_wrapping(xr, yr));
    }

    pub fn clone_sub_wrapping(&self, xr: Range<usize>, yr: Range<usize>) -> Array2D<T> {
        let width = xr.end - xr.start;
        let height = yr.end - yr.start;
        let mut arr = Vec::with_capacity(width * height);
        self.for_each_sub_wrapping(xr, yr, |_, _, t| arr.push(*t));

        return Array2D::from_box(width, height, arr.into_boxed_slice());
    }

    pub fn splice_wrapping(&mut self, xr: Range<usize>, yr: Range<usize>, data: Box<[T]>) {
        let mut i = 0;
        self.for_each_sub_wrapping_mut(xr, yr, |_, _, t| {
            *t = data[i];
            i += 1;
        });
    }

    pub fn for_each_sub_wrapping(
        &self,
        xr: Range<usize>,
        yr: Range<usize>,
        mut f: impl FnMut(usize, usize, &T),
    ) {
        let (w, h) = self.size();
        for_each_sub_wrapping(w, h, xr, yr, |x, y, index| {
            f(x, y, &self.data[index]);
        });
    }

    pub fn for_each_sub_wrapping_mut(
        &mut self,
        xr: Range<usize>,
        yr: Range<usize>,
        mut f: impl FnMut(usize, usize, &mut T),
    ) {
        let (w, h) = self.size();
        for_each_sub_wrapping(w, h, xr, yr, |x, y, index| {
            f(x, y, &mut self.data[index]);
        });
    }
}
