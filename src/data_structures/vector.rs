use std::ops::{Add, Index, IndexMut, Mul, Sub};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Default, bytemuck::Zeroable)]
pub struct Vec3<T>(pub T, pub T, pub T);

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Default, bytemuck::Zeroable)]
pub struct Vec4<T>(pub T, pub T, pub T, pub T);

unsafe impl<T> bytemuck::Pod for Vec3<T> where T: bytemuck::Pod {}
unsafe impl<T> bytemuck::Pod for Vec4<T> where T: bytemuck::Pod {}

pub type Vec3f32 = Vec3<f32>;
pub type Vec3u32 = Vec3<u32>;
pub type Vec4f32 = Vec4<f32>;
pub type Vec4u32 = Vec4<u32>;

#[inline(always)]
pub const fn vec3f32(f0: f32, f1: f32, f2: f32) -> Vec3<f32> {
    Vec3::<f32>(f0, f1, f2)
}
#[inline(always)]
pub const fn vec3f(f0: f32, f1: f32, f2: f32) -> Vec3<f32> {
    vec3f32(f0, f1, f2)
}

#[inline(always)]
pub const fn vec3u32(u0: u32, u1: u32, u2: u32) -> Vec3<u32> {
    Vec3::<u32>(u0, u1, u2)
}

/// Vec3 Methods
///

impl<T> Vec3<T>
where
    T: Default + Copy,
{
    pub fn vec4(&self) -> Vec4<T> {
        Vec4::<T>(self.0, self.1, self.2, Default::default())
    }
}

impl<T> Add<Vec3<T>> for Vec3<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T> Sub<Vec3<T>> for Vec3<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T: Mul<Output = T>> Mul<Vec3<T>> for Vec3<T>
where
    T: Mul,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
    }
}

impl<T> Mul<T> for Vec3<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

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

impl<T> From<[T; 3]> for Vec3<T>
where
    T: Copy,
{
    fn from(value: [T; 3]) -> Self {
        Self(value[0], value[1], value[2])
    }
}

/// Vec4 Methods
///

impl<T> Vec4<T>
where
    T: Copy,
{
    pub fn xyz(&self) -> Vec3<T> {
        Vec3::<T>(self.0, self.1, self.2)
    }
}

impl<T> Add<Vec4<T>> for Vec4<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(
            self.0 + rhs.0,
            self.1 + rhs.1,
            self.2 + rhs.2,
            self.3 + rhs.3,
        )
    }
}

impl<T> Sub<Vec4<T>> for Vec4<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(
            self.0 - rhs.0,
            self.1 - rhs.1,
            self.2 - rhs.2,
            self.3 - rhs.3,
        )
    }
}

impl<T: Mul<Output = T>> Mul<Vec4<T>> for Vec4<T>
where
    T: Mul,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(
            self.0 * rhs.0,
            self.1 * rhs.1,
            self.2 * rhs.2,
            self.3 * rhs.3,
        )
    }
}

impl<T> Mul<T> for Vec4<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs, self.3 * rhs)
    }
}

impl<T> Index<u32> for Vec4<T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            3 => &self.3,
            _ => panic!("Unexpected index {index}"),
        }
    }
}

impl<T> IndexMut<u32> for Vec4<T> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            3 => &mut self.3,
            _ => panic!("Unexpected index {index}"),
        }
    }
}

impl<T> From<(T, T, T, T)> for Vec4<T> {
    fn from(value: (T, T, T, T)) -> Self {
        Self(value.0, value.1, value.2, value.3)
    }
}

impl<T> From<[T; 4]> for Vec4<T>
where
    T: Copy,
{
    fn from(value: [T; 4]) -> Self {
        Self(value[0], value[1], value[2], value[3])
    }
}
