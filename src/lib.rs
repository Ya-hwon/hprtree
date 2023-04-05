#![feature(test)]

extern crate test;

mod hprtree;

pub use crate::hprtree::*;

#[cfg(test)]
mod tests {
    use crate::{BBox, HPRTree, Point, HPRTreeBuilder};
    use test::Bencher;

    fn build_bench_hprtree(mult: usize) -> HPRTree<usize> {
        let mut index = HPRTreeBuilder::<usize>::new(259200);
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

    // #[bench]
    // fn hprtree_build_bench_large(b: &mut Bencher) {
    //     b.iter(|| {
    //         build_bench_hprtree(4);
    //     });
    // }

    // #[bench]
    // fn hprtree_build_bench_medium(b: &mut Bencher) {
    //     b.iter(|| {
    //         build_bench_hprtree(2);
    //     });
    // }

    #[bench]
    fn hprtree_build_bench_small(b: &mut Bencher) {
        b.iter(|| {
            build_bench_hprtree(1);
        });
    }

    #[bench]
    fn hprtree_query_bench_small(b: &mut Bencher) {
        let tree = build_bench_hprtree(1);
        b.iter(|| {
            for i in 0..9 {
                tree.query(&BBox {
                    minx: -10f32 * i as f32,
                    miny: -10f32 * i as f32,
                    maxx: 10f32 * i as f32,
                    maxy: 10f32 * i as f32,
                });
            }
        });
    }

    // #[bench]
    // fn hprtree_query_bench_medium(b: &mut Bencher) {
    //     let tree = build_bench_hprtree(2);
    //     b.iter(||{
    //         for i in 0..9 {
    //             tree.query(&BBox {
    //                 minx: -10f32 * i as f32,
    //                 miny: -10f32 * i as f32,
    //                 maxx: 10f32 * i as f32,
    //                 maxy: 10f32 * i as f32
    //             });
    //         }
    //     });
    // }

    // #[bench]
    // fn hprtree_query_bench_large(b: &mut Bencher) {
    //     let tree = build_bench_hprtree(4);
    //     b.iter(||{
    //         for i in 0..9 {
    //             tree.query(&BBox {
    //                 minx: -10f32 * i as f32,
    //                 miny: -10f32 * i as f32,
    //                 maxx: 10f32 * i as f32,
    //                 maxy: 10f32 * i as f32
    //             });
    //         }
    //     });
    // }

    #[test]
    fn hprtree_end_to_end() {
        let mut index = HPRTreeBuilder::new(259200);
        let mut x = -180f32;
        for i in 0..(180 * 2 * 2) {
            let mut y = -90f32;
            for j in 0..(90 * 2 * 2) {
                index.insert(i * 1000 + j, Point { x, y });
                y += 0.5;
            }
            x += 0.5;
        }
        let index = index.build();
        let list = index.query(&BBox {
            minx: -10f32,
            miny: -10f32,
            maxx: 10f32,
            maxy: 10f32,
        });
        assert!(list.len() == 1681);
        for elem in list {
            let j = elem % 1000;
            let i = (elem - j) / 1000;
            assert!(j <= 200);
            assert!(j >= 160);
            assert!(i <= 380);
            assert!(i >= 340);
        }
    }
}
