///
/// Axis aligned bounding box
/// Adapted from Javascript/C++ code provided by Jeppe Revall Frisvad,
/// originally based on code by Nvidia, MIT License (2008-2010)

use super::vector::*;

///
/// ### Bounding Box
/// Axis aligned bounding box type
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Bbox {
    pub min: Vec3f32,
    pub max: Vec3f32,
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct BboxGpu {
    pub min: Vec3f32,
    _padding0: f32,
    pub max: Vec3f32,
    _padding1: f32,
}
static_assertions::assert_eq_size!(BboxGpu, [u8; 4*4*2]);

impl From<Bbox> for BboxGpu {
    fn from(value: Bbox) -> Self {
        Self {
            min: value.min,
            _padding0: 0.0,
            max: value.max,
            _padding1: 0.0,
        }
    }
}

/// Since this implementation mostly mirrors the optix one, we leave
/// the methods in even though we don't strictly need them
#[allow(dead_code)]
impl Bbox {
    ///
    /// Create a new bounding box including nothing
    pub fn new() -> Bbox {
        Self {
            min: vec3f( 1.0E+37, 1.0E+37, 1.0e+37),
            max: vec3f(-1.0e+37, -1.0e+37, -1.0e+37),
        }
    }

    ///
    /// Create a bounding box from a given triangle
    pub fn from_triangle(v0: Vec3f32, v1: Vec3f32, v2: Vec3f32) -> Bbox {
        Self {
        min: vec3f(f32::min(v0.0, f32::min(v1.0, v2.0)), f32::min(v0.1, f32::min(v1.1, v2.1)), f32::min(v0.2, f32::min(v1.2, v2.2))),
        max: vec3f(f32::max(v0.0, f32::max(v1.0, v2.0)), f32::max(v0.1, f32::max(v1.1, v2.1)), f32::max(v0.2, f32::max(v1.2, v2.2))),
        }
    }

    /// Extend the bounding box to include the given vertex
    pub fn include_vertex(&mut self, v: Vec3f32) {
        self.min.0 = f32::min(self.min.0, v.0);
        self.min.1 = f32::min(self.min.1, v.1);
        self.min.2 = f32::min(self.min.2, v.2);
        
        self.max.0 = f32::max(self.max.0, v.0);
        self.max.1 = f32::max(self.max.1, v.1);
        self.max.2 = f32::max(self.max.2, v.2);
    }

    /// Extend the bounding box to include the given bounding box
    pub fn include_bbox(&mut self, other: &Bbox) {
        self.min.0 = f32::min(self.min.0, other.min.0);
        self.min.1 = f32::min(self.min.1, other.min.1);
        self.min.2 = f32::min(self.min.2, other.min.2);

        self.max.0 = f32::max(self.max.0, other.max.0);
        self.max.1 = f32::max(self.max.1, other.max.1);
        self.max.2 = f32::max(self.max.2, other.max.2);
    }

    
    pub fn set_from_triangle(&mut self, v0: Vec3f32, v1: Vec3f32, v2: Vec3f32) {
        self.min = vec3f(f32::min(v0.0, f32::min(v1.0, v2.0)), f32::min(v0.1, f32::min(v1.1, v2.1)), f32::min(v0.2, f32::min(v1.2, v2.2)));
        self.max = vec3f(f32::max(v0.0, f32::max(v1.0, v2.0)), f32::max(v0.1, f32::max(v1.1, v2.1)), f32::max(v0.2, f32::max(v1.2, v2.2)));
    }

    /// Get the center of the bounding box
    pub fn center(&self) -> Vec3f32 {
        (self.min + self.max) * 0.5
    }

    /// Get the center of the given dimension of the bounding box
    pub fn center_dim(&self, dim: u32) -> f32 {
        self.max[dim] - self.min[dim]
    }

    /// Get the extents of the bounding box
    /// also called the diagonal
    pub fn extent(&self) -> Vec3f32 {
        self.max - self.min
    }

    /// Get the extent of the bounding box in the given dimenson
    pub fn extent_dim(&self, dim: u32) -> f32 {
        self.max[dim] - self.min[dim]
    }

    /// Get the volume of the bounding box
    pub fn volume(&self) -> f32 {
        let d = self.extent();
        d.0 * d.1 * d.2
    }

    /// Get the area of the bounding box
    pub fn area(&self) -> f32 {
        2.0 * self.half_area()
    }

    /// Get half of the area of the bounding box
    pub fn half_area(&self) -> f32 {
        let d = self.extent();
        d.0*d.1 + d.1*d.2 + d.2*d.0
    }

    /// Get the longest axis of the bounding box as an index
    pub fn longest_axis(&self) -> u32 {
        let d = self.extent();
        if d.0 > d.1 {
            if d.0 > d.2  {
                0
            } else {
                2
            }
        } else {
            if d.1 > d.2 {
                1
            } else {
                2
            }
        }
    }

    /// Get the largest extent of the bounding box
    pub fn max_extent(&self) -> f32 {
        self.extent_dim(self.longest_axis())
    }

    /// Check if the bounding box intersects with the other bounding box
    pub fn intersects(&self, other: &Bbox) -> bool {
        !(other.min.0 > self.max.0 || other.max.0 < self.min.0) &&
        !(other.min.1 > self.max.1 || other.max.1 < self.min.1) &&
        !(other.min.2 > self.max.2 || other.max.2 < self.min.2)
    }

    pub fn distance_center(&self, other: &Bbox) -> f32 {
        let center1 = self.center();
        let center2 = other.center();
        let distance = center1 - center2;
        distance.magnitude()
    }

    /// Offset function (from the PBR book)
    /// Return the relative position of a point inside of the box.
    /// 
    /// The minimum corner will have an offset of (0, 0, 0) and
    /// the maximum corner will have an offset of (1, 1, 1).
    pub fn offset(&self, point: Vec3f32) -> Vec3f32 {
        let mut o = point - self.min;
        if self.max.0 > self.min.0 {
            o.0 /= self.max.0 - self.min.0;
        }
        if self.max.1 > self.min.1 {
            o.1 /= self.max.1 - self.min.1;
        }
        if self.max.2 > self.min.2 {
            o.2 /= self.max.2 - self.min.2;
        }
        o
    }

}