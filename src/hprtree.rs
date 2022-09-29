use std::mem::size_of;

/// A simple stuct representing a bounding box / envelope, intended for lat/lon coordinates with lat=y, lon=x
///
/// Used for querying the index and for the internal data structure of the HPRTree
#[derive(Clone)]
pub struct BBox {
    pub minx: f32,
    pub miny: f32,
    pub maxx: f32,
    pub maxy: f32,
}

impl BBox {
    /// The default of the bbox is min = f32::MAX and max = f32::MIN
    ///
    /// This is so that "expanding to include"ing such a bbox results in whatever was used to expand the bbox by
    pub fn default() -> BBox {
        BBox {
            minx: f32::MAX,
            miny: f32::MAX,
            maxx: f32::MIN,
            maxy: f32::MIN,
        }
    }

    pub fn new(minx: f32, miny: f32, maxx: f32, maxy: f32) -> BBox {
        BBox {
            minx,
            miny,
            maxx,
            maxy,
        }
    }

    /// Returns the width of the bbox
    pub fn width(&self) -> f32 {
        self.maxx - self.minx
    }

    /// Returns the height of the bbox
    pub fn height(&self) -> f32 {
        self.maxy - self.miny
    }

    /// Expands the bbox to include another bbox
    pub fn expand_to_include(&mut self, other: &BBox) {
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

    /// Checks if a given point is contained within the bounds of the bbox
    pub fn contains(&self, other: &Point) -> bool {
        !(other.x > self.maxx || other.x < self.minx || other.y > self.maxy || other.y < self.miny)
    }

    /// Checks if a given bbox intersects the self bbox
    pub fn intersects(&self, other: &BBox) -> bool {
        !(other.minx > self.maxx
            || other.maxx < self.minx
            || other.miny > self.maxy
            || other.maxy < self.miny)
    }
}

/// A simple point struct, intended for lat/lon coordinates with lat=y, lon=x
#[derive(Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
struct IndexItem<T> {
    pub index_geom: Point,
    pub item: T,
}

const NODE_CAPACITY: usize = 16;
const HILBERT_LEVEL: usize = 12;
const H: usize = (1 << HILBERT_LEVEL) - 1;

/// The spatial index, start here
pub struct HPRTree<T> {
    items: Vec<IndexItem<T>>,
    extent: BBox,
    layer_start_index: Vec<usize>,
    node_bounds: Vec<BBox>,
}

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

    return ((interleave(i1) << 1) | interleave(i0)) >> (32 - 2 * HILBERT_LEVEL);
}

/// Example usage:
///
/// ```
/// use hprtree::{Point, BBox, HPRTree};
///
/// let mut index = HPRTree::new(10);
/// index.insert("Bob".to_string(), &Point{ x: 0f32, y: 0f32 });
/// for _ in 0..2 {
///     index.insert("Alice".to_string(), &Point{ x: 1f32, y: 1f32 });
/// }
/// index.insert("James".to_string(), &Point{ x: 2.5f32, y: -2.5f32 });
/// index.insert("Annie".to_string(), &Point{ x: 20f32, y: 1f32 });
/// for _ in 0..5 {
///     index.insert("Thomas".to_string(), &Point{ x: 1f32, y: -50f32 });
/// }
///
/// index.build();
///
/// let mut result = Vec::with_capacity(4);
/// index.query_with_list(&BBox
///            {
///                minx: -5f32,
///                miny: -5f32,
///                maxx: 5f32,
///                maxy: 5f32
///            }, &mut result);
///
/// assert!(result.len() == 4);// this Vec now contains the Strings "Bob", "Alice"(x2) and "James"
/// for i in result {
///     assert!(i == "Bob".to_string() || i == "Alice".to_string() || i == "James".to_string());
///     // there are absolutely no guarantees regarding ordering though
/// }
/// ```

impl<T> HPRTree<T>
where
    T: Clone,
{
    fn get_layer_size(&self, layer: usize) -> usize {
        self.layer_start_index[layer + 1] - self.layer_start_index[layer]
    }

    fn sort_items(&mut self) {
        let stride_x = self.extent.width() / H as f32;
        let stride_y = self.extent.height() / H as f32;

        let extent_min_x = self.extent.minx;

        self.items.sort_unstable_by(|lhs, rhs| {
            let xlhs: u32 = ((lhs.index_geom.x - extent_min_x) / stride_x).trunc() as u32;
            let ylhs: u32 = ((lhs.index_geom.y - extent_min_x) / stride_y).trunc() as u32;

            let xrhs: u32 = ((rhs.index_geom.x - extent_min_x) / stride_x).trunc() as u32;
            let yrhs: u32 = ((rhs.index_geom.y - extent_min_x) / stride_y).trunc() as u32;

            let indexlhs = hilbert_xy_to_index(xlhs, ylhs);
            let indexrhs = hilbert_xy_to_index(xrhs, yrhs);

            indexlhs.cmp(&indexrhs)
        });
    }

    fn compute_layer_start_indices(&mut self) {
        let mut item_count = self.items.len();
        self.layer_start_index =
            Vec::with_capacity((item_count as f32).log(NODE_CAPACITY as f32).trunc() as usize);
        let mut index: usize = 0;

        loop {
            self.layer_start_index.push(index);

            item_count /= NODE_CAPACITY;

            if item_count * NODE_CAPACITY != item_count {
                item_count += 1;
            }
            index += item_count;

            if item_count <= 1 {
                break;
            }
        }
    }

    fn compute_leaf_nodes(&mut self) {
        for i in 0..self.layer_start_index[1] {
            for j in 0..=NODE_CAPACITY {
                let index = NODE_CAPACITY * i + j;
                if index >= self.items.len() {
                    return;
                }
                self.node_bounds[i].expand_to_include_point(&self.items[index].index_geom);
            }
        }
    }
    fn compute_layer_nodes(&mut self) {
        for i in 1..(self.layer_start_index.len() - 1) {
            let layer_start = self.layer_start_index[i];
            let layer_size = self.get_layer_size(i);
            let child_layer_start = self.layer_start_index[i - 1];
            let child_layer_end = layer_start;
            for j in 0..layer_size {
                let child_start = child_layer_start + NODE_CAPACITY * j;
                for k in 0..=NODE_CAPACITY {
                    let index = child_start + k;
                    if index >= child_layer_end {
                        break;
                    }
                    let child = self.node_bounds[index].clone();
                    self.node_bounds[layer_start + j].expand_to_include(&child);
                }
            }
        }
    }

    fn query_node_children(
        &self,
        layer_index: usize,
        block_offset: &usize,
        query_env: &BBox,
        candidate_list: &mut Vec<T>,
    ) {
        let layer_start = self.layer_start_index[layer_index];
        let layer_end = self.layer_start_index[layer_index + 1];
        for i in 0..NODE_CAPACITY {
            let node_offset = block_offset + i;
            if node_offset + layer_start >= layer_end {
                return;
            }
            self.query_node(&layer_index, &node_offset, query_env, candidate_list)
        }
    }

    fn query_items(&self, block_start: usize, query_env: &BBox, candidate_list: &mut Vec<T>) {
        for i in 0..NODE_CAPACITY {
            let item_index = block_start + i;
            if item_index >= self.items.len() {
                return;
            }
            let current_item = self.items[item_index].clone();
            if query_env.contains(&current_item.index_geom) {
                candidate_list.push(current_item.item);
            }
        }
    }

    fn query_node(
        &self,
        layer_index: &usize,
        node_offset: &usize,
        query_env: &BBox,
        candidate_list: &mut Vec<T>,
    ) {
        let layer_start = self.layer_start_index[*layer_index];
        let node_index = layer_start + *node_offset;

        if !query_env.intersects(&self.node_bounds[node_index]) {
            return;
        }
        let child_node_offset = node_offset * NODE_CAPACITY;
        if *layer_index != 0 {
            self.query_node_children(
                *layer_index - 1,
                &child_node_offset,
                query_env,
                candidate_list,
            );
        } else {
            self.query_items(child_node_offset, query_env, candidate_list);
        }
    }

    /// Queries the tree by bounding box and pushes the found elements onto the vector, useful if the usecase enables better estimates for how many elements will be found (to reduce the chance for reallocation, over or underallocation)
    ///
    /// Only query after the tree is built!
    pub fn query_with_list(&self, query_env: &BBox, candidate_list: &mut Vec<T>) {
        if !self.extent.intersects(query_env) {
            return;
        }

        if self.layer_start_index.len() == 0 {
            self.query_items(0, query_env, candidate_list);
            return;
        }

        let layer_index = self.layer_start_index.len() - 2;
        let layer_size = self.get_layer_size(layer_index);

        for i in 0..layer_size {
            self.query_node(&layer_index, &i, query_env, candidate_list);
        }
    }

    /// Queries the tree by bounding box returning a Vec of the found elements
    ///
    /// Only query after the tree is built!
    pub fn query(&self, query_env: &BBox) -> Vec<T> {
        if !self.extent.intersects(query_env) {
            return Vec::new();
        }

        let mut candidate_list = Vec::with_capacity(
            (self.avg_entries() * query_env.height() * query_env.width()) as usize,
        );

        self.query_with_list(query_env, &mut candidate_list);

        candidate_list
    }

    /// Creates a new tree with base capacity
    pub fn new(size: usize) -> Self {
        HPRTree {
            items: Vec::with_capacity(size),
            extent: BBox::default(),
            layer_start_index: Vec::new(),
            node_bounds: Vec::new(),
        }
    }

    /// Returns how many elements are in an area unit on average, may help with guessing how many entities will be found in a given bounding box if the entries are somewhat evenly destributed
    pub fn avg_entries(&self) -> f32 {
        self.items.len() as f32 / (self.extent.height() * self.extent.width())
    }

    /// Returns how many bytes are taken up by the trees data
    pub fn current_size_in_bytes(&self) -> usize {
        self.items.len() * size_of::<IndexItem<T>>()
            + self.layer_start_index.len() * size_of::<usize>()
            + self.node_bounds.len() * size_of::<BBox>()
            + size_of::<BBox>()
    }

    /// Inserts an element into the index
    ///
    /// The elements are not to be queried for until the tree has been built
    pub fn insert(&mut self, item: T, geom: &Point) {
        self.items.push(IndexItem {
            index_geom: geom.clone(),
            item: item,
        });
        self.extent.expand_to_include_point(geom);
    }

    /// Builds the index, making previously inserted elements available through query
    pub fn build(&mut self) {
        if self.items.len() < NODE_CAPACITY {
            return;
        }

        self.sort_items();

        self.compute_layer_start_indices();

        self.node_bounds =
            vec![BBox::default(); self.layer_start_index[self.layer_start_index.len() - 1]];

        self.compute_leaf_nodes();
        self.compute_layer_nodes();
    }

    /// Returns the number of elements in the tree
    pub fn size(&self) -> usize {
        self.items.len()
    }
}
