pub mod testutil {

    use hprtree::{HPRTree, HPRTreeBuilder, Point};

    pub fn build_regular_hprtree(mult: usize) -> HPRTree<usize> {
        let expected_size = mult * 180 * mult * 90;
        let mut index = HPRTreeBuilder::<usize>::new(expected_size);
        let mut x = -180f32;
        for i in 0..(180 * mult) {
            let mut y = -90f32;
            for j in 0..(90 * mult) {
                index.insert(i * 1000 + j, Point { x, y });
                y += 2f32 / mult as f32;
            }
            x += 2f32 / mult as f32;
        }
        index.build()
    }
}
