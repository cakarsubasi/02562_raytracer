use std::ops::{Add, Index, Mul, Sub};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3<T>(pub T, pub T, pub T);

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

pub type Vec3f32 = Vec3<f32>;
#[inline(always)]
pub fn vec3f32(f0: f32, f1: f32, f2: f32) -> Vec3<f32> {
    Vec3::<f32>(f0, f1, f2)
}
#[inline(always)]
pub fn vec3f(f0: f32, f1: f32, f2: f32) -> Vec3<f32> {
    Vec3::<f32>(f0, f1, f2)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Vec4u32(pub u32, pub u32, pub u32, pub u32);
