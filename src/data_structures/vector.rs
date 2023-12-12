use std::ops::{Add, Index, IndexMut, Mul, Sub, Div, Not};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Default, bytemuck::Zeroable)]
pub struct Vec3<T>(pub T, pub T, pub T);

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Default, bytemuck::Zeroable)]
pub struct Vec4<T>(pub T, pub T, pub T, pub T);

unsafe impl<T> bytemuck::Pod for Vec3<T> where T: bytemuck::Pod {}
unsafe impl<T> bytemuck::Pod for Vec4<T> where T: bytemuck::Pod {}

pub type Vec3f32 = Vec3<f32>;
#[allow(dead_code)]
pub type Vec3u32 = Vec3<u32>;
pub type Vec4f32 = Vec4<f32>;
pub type Vec4u32 = Vec4<u32>;
#[allow(dead_code)]
pub type Point3<T> = Vec3<T>;
#[allow(dead_code)]
pub type Point4<T> = Vec4<T>;

#[inline(always)]
pub const fn vec3f32(f0: f32, f1: f32, f2: f32) -> Vec3<f32> {
    Vec3::<f32>(f0, f1, f2)
}
#[inline(always)]
pub const fn vec3f(f0: f32, f1: f32, f2: f32) -> Vec3<f32> {
    vec3f32(f0, f1, f2)
}
#[inline(always)]
pub const fn vec4f32(f0: f32, f1: f32, f2: f32, f3: f32) -> Vec4<f32> {
    Vec4::<f32>(f0, f1, f2, f3)
}

#[inline(always)]
pub const fn vec3u32(u0: u32, u1: u32, u2: u32) -> Vec3<u32> {
    Vec3::<u32>(u0, u1, u2)
}

#[inline(always)]
pub const fn vec4u32(u0: u32, u1: u32, u2: u32, u3: u32) -> Vec4<u32> {
    Vec4::<u32>(u0, u1, u2, u3)
}

pub fn dot<T>(v1: Vec3<T>, v2: Vec3<T>) -> T 
where T: Mul<Output = T> + Add<Output = T>
{
    v1.0 * v2.0 + v1.1 * v2.1 + v1.2 * v2.2
}

/// Vec3 Methods
///

impl<T> Vec3<T>
where T: Default
{
    pub fn vec4(self) -> Vec4<T> {
        Vec4::<T>(self.0, self.1, self.2, Default::default())
    }
}

pub trait Sqrt {
    type Output;
    fn sqrt(self) -> Self::Output;
}

impl Sqrt for Vec3<f32> {
    type Output = Vec3<f32>;

    fn sqrt(self) -> Self::Output {
        vec3f32(self.0.sqrt(), self.1.sqrt(), self.2.sqrt())
    }
}

impl Sqrt for f32 {

    fn sqrt(self) -> Self::Output {
        self.sqrt()
    }

    type Output = f32;
}

impl<T> Vec3<T>
where T: Copy + Sqrt<Output = T> + Mul<Output = T> + Add<Output = T> + Div<Output = T>,
//Vec3<T>: Div<Output = Vec3<T>> 
{
    pub fn magnitude(self) -> T {
        T::sqrt(self.0 * self.0 + self.1 * self.1 + self.2 * self.2)
    }

    pub fn normalize(self) -> Self {
        let magnitude = self.magnitude();
        Self(self.0 / magnitude, self.1 / magnitude, self.2 / magnitude)
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

impl<T> Div<T> for Vec3<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs, self.2 / rhs)
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

// comparisons

impl<T> Vec3<T> where T: PartialOrd<T> {
    pub fn lt(&self, rhs: Vec3<T>) -> Vec3<bool> {
        let x = self.0 < rhs.0;
        let y = self.1 < rhs.1;
        let z = self.2 < rhs.2;
        Vec3::<bool>(x, y, z)
    }

    pub fn le(&self, rhs: Vec3<T>) -> Vec3<bool> {
        let x = self.0 <= rhs.0;
        let y = self.1 <= rhs.1;
        let z = self.2 <= rhs.2;
        Vec3::<bool>(x, y, z)
    }

    pub fn gt(&self, rhs: Vec3<T>) -> Vec3<bool> {
        let x = self.0 > rhs.0;
        let y = self.1 > rhs.1;
        let z = self.2 > rhs.2;
        Vec3::<bool>(x, y, z)
    }

    pub fn ge(&self, rhs: Vec3<T>) -> Vec3<bool> {
        let x = self.0 >= rhs.0;
        let y = self.1 >= rhs.1;
        let z = self.2 >= rhs.2;
        Vec3::<bool>(x, y, z)
    }
}

impl<T> Vec3<T> where T: Into<bool> + Copy {
    pub fn all(&self) -> bool {
        self.0.into() && self.1.into() && self.2.into()
    }

    pub fn any(&self) -> bool {
        self.0.into() || self.1.into() || self.2.into()
    }
}

impl<T> Not for Vec3<T> where T: Into<bool> {
    type Output = Vec3<bool>;

    fn not(self) -> Self::Output {
        Vec3::<bool>(!self.0.into(), !self.1.into(), !self.2.into())
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
