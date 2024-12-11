mod hprtree;
pub use crate::hprtree::*;
mod hprtree_wrapping;
pub use crate::hprtree_wrapping::*;

/// A simple stuct representing a bounding box / envelope, intended for lat/lon coordinates with lat=y, lon=x
///
/// Used for querying the index and for the internal data structure of the HPRTree
#[derive(Clone, Debug)]
pub struct BBox {
    pub minx: CoordinateType,
    pub miny: CoordinateType,
    pub maxx: CoordinateType,
    pub maxy: CoordinateType,
}

impl Default for BBox {
    /// The default of the bbox is min = f32::MAX and max = f32::MIN
    ///
    /// This is so that "expanding to include"ing such a bbox results in whatever was used to expand the bbox by
    fn default() -> Self {
        Self {
            minx: CoordinateType::MAX,
            miny: CoordinateType::MAX,
            maxx: CoordinateType::MIN,
            maxy: CoordinateType::MIN,
        }
    }
}

impl BBox {
    pub fn new(
        minx: CoordinateType,
        miny: CoordinateType,
        maxx: CoordinateType,
        maxy: CoordinateType,
    ) -> Self {
        Self {
            minx,
            miny,
            maxx,
            maxy,
        }
    }

    /// Returns the width of the bbox
    pub fn width(&self) -> CoordinateType {
        self.maxx - self.minx
    }

    /// Returns the height of the bbox
    pub fn height(&self) -> CoordinateType {
        self.maxy - self.miny
    }

    /// Expands the bbox to include another bbox
    pub fn expand_to_include(&mut self, other: &Self) {
        self.minx = self.minx.min(other.minx);
        self.miny = self.miny.min(other.miny);
        self.maxx = self.maxx.max(other.maxx);
        self.maxy = self.maxy.max(other.maxy);
    }

    /// Expands the bbox to include a point
    pub fn expand_to_include_point(&mut self, point: &Point) {
        self.minx = self.minx.min(point.x);
        self.miny = self.miny.min(point.y);
        self.maxx = self.maxx.max(point.x);
        self.maxy = self.maxy.max(point.y);
    }

    /// Expands the bbox to include a point
    pub fn expand_to_include_spatially_indexable(&mut self, point: &impl SpatiallyIndexable) {
        self.minx = self.minx.min(point.x());
        self.miny = self.miny.min(point.y());
        self.maxx = self.maxx.max(point.x());
        self.maxy = self.maxy.max(point.y());
    }

    /// Checks if a given point is contained within the bounds of the bbox
    pub fn contains(&self, other: &Point) -> bool {
        !(other.x > self.maxx || other.x < self.minx || other.y > self.maxy || other.y < self.miny)
    }

    /// Checks if a given point is contained within the bounds of the bbox
    pub fn contains_spatially_indexable(&self, other: &impl SpatiallyIndexable) -> bool {
        !(other.x() > self.maxx
            || other.x() < self.minx
            || other.y() > self.maxy
            || other.y() < self.miny)
    }

    /// Checks if a given bbox intersects the self bbox
    pub fn intersects(&self, other: &Self) -> bool {
        !(other.minx > self.maxx
            || other.maxx < self.minx
            || other.miny > self.maxy
            || other.maxy < self.miny)
    }
}

/// A simple point struct, intended for lat/lon coordinates with lat=y, lon=x
#[derive(Clone, Debug)]
pub struct Point {
    pub x: CoordinateType,
    pub y: CoordinateType,
}

impl SpatiallyIndexable for Point {
    fn x(&self) -> CoordinateType {
        self.x
    }

    fn y(&self) -> CoordinateType {
        self.y
    }
}

/// Trait that enables a struct to be spatially indexed
pub trait SpatiallyIndexable {
    fn x(&self) -> CoordinateType;
    fn y(&self) -> CoordinateType;
}

const NODE_CAPACITY: usize = 16;
const HILBERT_LEVEL: usize = 12;
const H: usize = (1 << HILBERT_LEVEL) - 1;

fn interleave(x: u32) -> u32 {
    let x = (x | (x << 8)) & 0x00FF00FF;
    let x = (x | (x << 4)) & 0x0F0F0F0F;
    let x = (x | (x << 2)) & 0x33333333;
    (x | (x << 1)) & 0x55555555
}

#[allow(non_snake_case)]
fn hilbert_xy_to_index(x: u32, y: u32) -> u32 {
    let x = x << (16 - HILBERT_LEVEL);
    let y = y << (16 - HILBERT_LEVEL);

    let mut A: u32;
    let mut B: u32;
    let mut C: u32;
    let mut D: u32;

    // Initial prefix scan round, prime with x and y
    {
        let a = x ^ y;
        let b = 0xFFFF ^ a;
        let c = 0xFFFF ^ (x | y);
        let d = x & (y ^ 0xFFFF);

        A = a | (b >> 1);
        B = (a >> 1) ^ a;

        C = ((c >> 1) ^ (b & (d >> 1))) ^ c;
        D = ((a & (c >> 1)) ^ (d >> 1)) ^ d;
    }

    {
        let a = A;
        let b = B;
        let c = C;
        let d = D;

        A = (a & (a >> 2)) ^ (b & (b >> 2));
        B = (a & (b >> 2)) ^ (b & ((a ^ b) >> 2));

        C ^= (a & (c >> 2)) ^ (b & (d >> 2));
        D ^= (b & (c >> 2)) ^ ((a ^ b) & (d >> 2));
    }

    {
        let a = A;
        let b = B;
        let c = C;
        let d = D;

        A = (a & (a >> 4)) ^ (b & (b >> 4));
        B = (a & (b >> 4)) ^ (b & ((a ^ b) >> 4));

        C ^= (a & (c >> 4)) ^ (b & (d >> 4));
        D ^= (b & (c >> 4)) ^ ((a ^ b) & (d >> 4));
    }

    // Final round and projection
    {
        let a = A;
        let b = B;
        let c = C;
        let d = D;

        C ^= (a & (c >> 8)) ^ (b & (d >> 8));
        D ^= (b & (c >> 8)) ^ ((a ^ b) & (d >> 8));
    }

    // Undo transformation prefix scan
    let a = C ^ (C >> 1);
    let b = D ^ (D >> 1);

    // Recover index bits
    let i0 = x ^ y;
    let i1 = b | (0xFFFF ^ (i0 | a));

    ((interleave(i1) << 1) | interleave(i0)) >> (32 - 2 * HILBERT_LEVEL)
}

fn get_layer_size(layer: usize, layer_start_index: &[usize]) -> usize {
    layer_start_index[layer + 1] - layer_start_index[layer]
}

/// Internal type for coordinates
pub type CoordinateType = f32;
