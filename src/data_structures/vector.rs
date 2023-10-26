use std::ops::{Add, Index, Mul, Sub, IndexMut};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Default, bytemuck::Zeroable)]
pub struct Vec3<T>(pub T, pub T, pub T);

unsafe impl bytemuck::Pod for Vec3<f32> {}
unsafe impl bytemuck::Pod for Vec3<f64> {}
unsafe impl bytemuck::Pod for Vec3<u32> {}
unsafe impl bytemuck::Pod for Vec3<u64> {}


impl<T> Add<Vec3<T>> for Vec3<T>
where
    T: Add<Output = T>,
{
    type Output = Vec3<T>;

    fn add(self, rhs: Vec3<T>) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T> Sub<Vec3<T>> for Vec3<T>
where
    T: Sub<Output = T>,
{
    type Output = Vec3<T>;

    fn sub(self, rhs: Vec3<T>) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T: Mul<Output = T>> Mul<Vec3<T>> for Vec3<T>
where
    T: Mul,
{
    type Output = Vec3<T>;

    fn mul(self, rhs: Vec3<T>) -> Self::Output {
        Self(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
    }
}

impl<T> Mul<T> for Vec3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec3<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl<T> Index<u32> for Vec3<T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("Unexpected index {index}"),
        }
    }
}

impl<T> IndexMut<u32> for Vec3<T> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("Unexpected index {index}"),
        }
    }
}

impl<T> From<(T, T, T)> for Vec3<T> {
    fn from(value: (T, T, T)) -> Self {
        Vec3::<T>(value.0, value.1, value.2)
    }
}

impl<T> From<[T; 3]> for Vec3<T> where T: Copy {
    fn from(value: [T; 3]) -> Self {
        Vec3::<T>(value[0], value[1], value[2])
    }
}

pub type Vec3f32 = Vec3<f32>;
pub type Vec3u32 = Vec3<u32>;
#[inline(always)]
pub fn vec3f32(f0: f32, f1: f32, f2: f32) -> Vec3<f32> {
    Vec3::<f32>(f0, f1, f2)
}
#[inline(always)]
pub fn vec3f(f0: f32, f1: f32, f2: f32) -> Vec3<f32> {
    vec3f32(f0, f1, f2)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec4u32(pub u32, pub u32, pub u32, pub u32);