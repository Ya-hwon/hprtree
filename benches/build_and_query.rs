#![feature(test)]
extern crate test;

use hprtree::{BBox, HPRTree, HPRTreeBuilder, Point};
use test::Bencher;

fn build_bench_hprtree(mult: usize) -> HPRTree<usize> {
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

#[bench]
fn hprtree_build_bench_small(b: &mut Bencher) {
    b.iter(|| {
        build_bench_hprtree(1);
    });
}

#[bench]
fn hprtree_build_bench_medium(b: &mut Bencher) {
    b.iter(|| {
        build_bench_hprtree(2);
    });
}

#[bench]
fn hprtree_build_bench_large(b: &mut Bencher) {
    b.iter(|| {
        build_bench_hprtree(4);
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

#[bench]
fn hprtree_query_bench_medium(b: &mut Bencher) {
    let tree = build_bench_hprtree(2);
    b.iter(||{
        for i in 0..9 {
            tree.query(&BBox {
                minx: -10f32 * i as f32,
                miny: -10f32 * i as f32,
                maxx: 10f32 * i as f32,
                maxy: 10f32 * i as f32
            });
        }
    });
}

#[bench]
fn hprtree_query_bench_large(b: &mut Bencher) {
    let tree = build_bench_hprtree(4);
    b.iter(||{
        for i in 0..9 {
            tree.query(&BBox {
                minx: -10f32 * i as f32,
                miny: -10f32 * i as f32,
                maxx: 10f32 * i as f32,
                maxy: 10f32 * i as f32
            });
        }
    });
}