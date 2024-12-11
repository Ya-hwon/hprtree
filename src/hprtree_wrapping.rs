use std::mem::size_of;

use crate::{get_layer_size, hilbert_xy_to_index, BBox, Point, H, NODE_CAPACITY};

#[derive(Clone)]
struct IndexItem<T>
where
    T: Clone,
{
    pub index_geom: Point,
    pub item: T,
}

/// The builder for the spatial index variant
#[derive(Clone)]
pub struct HPRTreeWrappingBuilder<T>
where
    T: Clone,
{
    items: Vec<IndexItem<T>>,
    extent: BBox,
}

/// A variant of the spatial index which takes the element and its geometry in separately and does not return the geometry from queries
pub struct HPRTreeWrapping<T>
where
    T: Clone,
{
    items: Vec<IndexItem<T>>,
    extent: BBox,
    layer_start_index: Vec<usize>,
    node_bounds: Vec<BBox>,
}

/// Example usage:
///
/// ```
/// use hprtree::{Point, BBox, HPRTreeWrappingBuilder};
///
/// let mut index = HPRTreeWrappingBuilder::new(10);
/// index.insert("Bob", Point{ x: 0f32, y: 0f32 });
/// for _ in 0..2 {
///     index.insert("Alice", Point{ x: 1f32, y: 1f32 });
/// }
/// index.insert("James", Point{ x: 2.5f32, y: -2.5f32 });
/// index.insert("Annie", Point{ x: 20f32, y: 1f32 });
/// for _ in 0..5 {
///     index.insert("Thomas", Point{ x: 1f32, y: -50f32 });
/// }
///
/// let index = index.build();
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
/// assert!(result.len() == 4);// this Vec now contains the &strs "Bob", "Alice"(x2) and "James"
/// for i in result {
///     assert!(i == "Bob" || i == "Alice" || i == "James");
///     // there are absolutely no guarantees regarding ordering though
/// }
/// ```

impl<T> HPRTreeWrappingBuilder<T>
where
    T: Clone,
{
    /// Creates a new tree builder with base capacity
    pub fn new(size: usize) -> Self {
        HPRTreeWrappingBuilder {
            items: Vec::with_capacity(size),
            extent: BBox::default(),
        }
    }

    /// Inserts an element into the index
    pub fn insert(&mut self, item: T, geom: Point) {
        self.extent.expand_to_include_point(&geom);
        self.items.push(IndexItem {
            index_geom: geom,
            item,
        });
    }

    /// Sorts the data, builds the index and transfers the builders state into an HPRTree which is then returned. If [sort_items](#method.sort_items) has been called before, prefer [build_sorted](#method.build_sorted) instead
    pub fn build(mut self) -> HPRTreeWrapping<T> {
        if self.items.len() < NODE_CAPACITY {
            return HPRTreeWrapping {
                items: self.items,
                extent: self.extent,
                layer_start_index: Vec::new(),
                node_bounds: Vec::new(),
            };
        }

        self.sort_items();

        self.build_sorted()
    }

    /// Sorts the contained data (in preparation for [build_sorted](#method.build_sorted))
    pub fn sort_items(&mut self) {
        let stride_x = if self.extent.width() != 0f32 {
            self.extent.width() / H as f32
        } else {
            1f32
        };
        let stride_y = if self.extent.height() != 0f32 {
            self.extent.height() / H as f32
        } else {
            1f32
        };

        let extent_min = self.extent.minx.min(self.extent.miny);

        self.items.sort_by_cached_key(|pt| {
            let x: u32 = ((pt.index_geom.x - extent_min) / stride_x).trunc() as u32;
            let y: u32 = ((pt.index_geom.y - extent_min) / stride_y).trunc() as u32;
            hilbert_xy_to_index(x, y)
        });
    }

    /// Builds the index and transfers the builders state into an HPRTree which is then returned, depends on the data being sorted (by [sort_items](#method.sort_items)) already
    pub fn build_sorted(self) -> HPRTreeWrapping<T> {
        if self.items.len() < NODE_CAPACITY {
            return HPRTreeWrapping {
                items: self.items,
                extent: self.extent,
                layer_start_index: Vec::new(),
                node_bounds: Vec::new(),
            };
        }

        let layer_start_index = self.compute_layer_start_indices();

        let mut node_bounds = vec![BBox::default(); *layer_start_index.last().unwrap()];

        self.compute_leaf_nodes(&layer_start_index, &mut node_bounds);
        self.compute_layer_nodes(&layer_start_index, &mut node_bounds);

        HPRTreeWrapping {
            items: self.items,
            extent: self.extent,
            layer_start_index,
            node_bounds,
        }
    }

    /// Returns the number of elements in the tree
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the tree is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the extent of the tree
    pub fn extent(&self) -> BBox {
        self.extent.clone()
    }

    fn compute_layer_start_indices(&self) -> Vec<usize> {
        let mut item_count = self.items.len();
        let mut layer_start_index =
            Vec::with_capacity((item_count as f32).log(NODE_CAPACITY as f32).trunc() as usize);
        let mut index: usize = 0;

        loop {
            layer_start_index.push(index);

            item_count /= NODE_CAPACITY;

            if item_count * NODE_CAPACITY != item_count {
                item_count += 1;
            }
            index += item_count;

            if item_count <= 1 {
                break;
            }
        }
        layer_start_index
    }

    fn compute_leaf_nodes(&self, layer_start_index: &[usize], node_bounds: &mut [BBox]) {
        for i in 0..layer_start_index[1] {
            for j in 0..=NODE_CAPACITY {
                let index = NODE_CAPACITY * i + j;
                if index >= self.items.len() {
                    return;
                }
                node_bounds[i].expand_to_include_point(&self.items[index].index_geom);
            }
        }
    }
    fn compute_layer_nodes(&self, layer_start_index: &[usize], node_bounds: &mut [BBox]) {
        for i in 1..(layer_start_index.len() - 1) {
            let layer_start = layer_start_index[i];
            let layer_size = get_layer_size(i, layer_start_index);
            let child_layer_start = layer_start_index[i - 1];
            let child_layer_end = layer_start;
            for j in 0..layer_size {
                let child_start = child_layer_start + NODE_CAPACITY * j;
                for k in 0..=NODE_CAPACITY {
                    let index = child_start + k;
                    if index >= child_layer_end {
                        break;
                    }
                    let (node_bounds_left, node_bounds_right) =
                        node_bounds.split_at_mut(layer_start + j);

                    if let Some(child) = node_bounds_left.get(index) {
                        node_bounds_right[0].expand_to_include(child);
                    }
                    // this is ugly but arguably less ugly then the following - i sincerely hope there is a better way to do this though
                    // unsafe {
                    //     let child = node_bounds.as_ptr().offset(index.try_into().unwrap());
                    //     node_bounds[layer_start + j].expand_to_include(&*child);
                    // }
                }
            }
        }
    }
}

impl<T> HPRTreeWrapping<T>
where
    T: Clone,
{
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
            let current_item = &self.items[item_index];
            if query_env.contains(&current_item.index_geom) {
                candidate_list.push(current_item.item.clone());
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

    /// Queries the tree by bounding box returning a Vec of the found elements
    pub fn query(&self, query_env: &BBox) -> Vec<T> {
        if !self.extent.intersects(query_env) {
            return Vec::new();
        }

        let n_guessed_candidates =
            self.avg_entries() * query_env.height() * query_env.width() * 1.5;
        let mut candidate_list = Vec::with_capacity((n_guessed_candidates) as usize);

        self.query_with_list(query_env, &mut candidate_list);

        candidate_list
    }

    /// Queries the tree by bounding box and pushes the found elements onto the vector, useful if the usecase enables better estimates for how many elements will be found (to reduce the chance for reallocation or overallocation)
    pub fn query_with_list(&self, query_env: &BBox, candidate_list: &mut Vec<T>) {
        if !self.extent.intersects(query_env) {
            return;
        }

        if self.layer_start_index.is_empty() {
            self.query_items(0, query_env, candidate_list);
            return;
        }

        let layer_index = self.layer_start_index.len() - 2;
        let layer_size = get_layer_size(layer_index, &self.layer_start_index);

        for i in 0..layer_size {
            self.query_node(&layer_index, &i, query_env, candidate_list);
        }
    }

    /// Returns how many elements are in an area unit on average, may help with guessing how many entities will be found in a given bounding box if the entries are somewhat evenly distributed
    pub fn avg_entries(&self) -> f32 {
        let area = self.extent.height() * self.extent.width();
        if area == 0f32 {
            return self.items.len() as f32;
        }
        self.items.len() as f32 / area
    }

    /// Returns how many bytes are taken up by the trees data
    pub fn current_size_in_bytes(&self) -> usize {
        self.items.len() * size_of::<T>()
            + self.layer_start_index.len() * size_of::<usize>()
            + self.node_bounds.len() * size_of::<BBox>()
            + size_of::<Self>()
    }

    /// Approximates how many bytes would be taken up by the data of a tree with a given size and index element type
    pub fn projected_size_in_bytes(elems: usize) -> usize {
        elems * size_of::<T>()
            + (elems as f32).log(NODE_CAPACITY as f32).trunc() as usize * size_of::<usize>()
            + (elems as f64*0.0667+2.2143).trunc() as usize * size_of::<BBox>()
            // approximate linear regression from the following values
            // 16200    64800   145800  259200  405000  583200  793800  1036800
            // 1082     4323    9722    17283   27002   38882   52921   69124
            + size_of::<Self>()
    }

    /// Returns the number of elements in the tree
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the tree is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the extent of the tree
    pub fn extent(&self) -> BBox {
        self.extent.clone()
    }
}
