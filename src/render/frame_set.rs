use std::error::Error;
use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[repr(transparent)]
pub struct FrameSet<T>([T; MAX_FRAMES_IN_FLIGHT]);

impl<T> FrameSet<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.0.iter_mut()
    }


    pub fn create_factory<F: FnMut(usize) -> T>(f: F) -> Self {
        Self(std::array::from_fn(f))
    }
}

impl<T> Index<usize> for FrameSet<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for FrameSet<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T> IntoIterator for FrameSet<T> {
    type Item = T;
    type IntoIter = std::array::IntoIter<T, MAX_FRAMES_IN_FLIGHT>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T,E: Error> FrameSet<Result<T, E>> {
    pub fn promote_errors(self) -> Result<FrameSet<T>, E> {
        unsafe {
            let mut frame_set_uninit: [MaybeUninit<T>; MAX_FRAMES_IN_FLIGHT] = MaybeUninit::uninit().assume_init();

            for (i, elem) in self.0.into_iter().enumerate() {
                frame_set_uninit[i].write(elem?);
            }

            Ok(FrameSet(std::mem::transmute_copy::<_, [T; MAX_FRAMES_IN_FLIGHT]>(&frame_set_uninit)))
        }
    }
}

impl<T> Into<[T; MAX_FRAMES_IN_FLIGHT]> for FrameSet<T> {
    fn into(self) -> [T; MAX_FRAMES_IN_FLIGHT] {
        self.0
    }
}

impl<T> From<[T; MAX_FRAMES_IN_FLIGHT]> for FrameSet<T> {
    fn from(value: [T; MAX_FRAMES_IN_FLIGHT]) -> Self {
        Self(value)
    }
}

impl<T> From<Vec<T>> for FrameSet<T> where for <'a> &'a[T]: TryInto<[T; MAX_FRAMES_IN_FLIGHT]> {
    fn from(value: Vec<T>) -> Self {
        Self(value.as_slice()[..MAX_FRAMES_IN_FLIGHT].try_into().ok().unwrap())
    }
}
