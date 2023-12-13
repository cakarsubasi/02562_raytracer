use super::bbox::Bbox;

/// Intermediate data structure to pass
/// indexed bounding boxes to the BSP Tree and BVH
///
/// Each index points towards the primitive (in this case the triangle)
/// in the index buffer that corresponds to the bbox
#[derive(Debug, Copy, Clone)]
pub struct AccObj {
    pub idx: u32,
    pub bbox: Bbox,
}

impl AccObj {
    pub fn new(idx: u32, bbox: Bbox) -> Self {
        Self { idx, bbox }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Split {
    AxisX = 0,
    AxisY = 1,
    AxisZ = 2,
}

impl From<u32> for Split {
    fn from(value: u32) -> Self {
        match value {
            0 => Split::AxisX,
            1 => Split::AxisY,
            2 => Split::AxisZ,
            _ => panic!("unexpected input {value}"),
        }
    }
}