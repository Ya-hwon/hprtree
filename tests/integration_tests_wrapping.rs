#![feature(float_next_up_down)]

use hprtree::{BBox, HPRTreeWrappingBuilder, Point};
use rand::prelude::Distribution;

#[test]
fn random_contain_test() {
    const N_INCLUDED: usize = 100_000;
    const N_EXCLUDED: usize = 50_000;

    let mut rng = rand::thread_rng();

    let included_x_range = -180f32..=180f32;
    let included_x = rand::distributions::Uniform::from(included_x_range);
    let included_y_range = -90f32..=90f32;
    let included_y = rand::distributions::Uniform::from(included_y_range);

    let excluded_x_low_range = -360f32..-180f32;
    let excluded_x_low = rand::distributions::Uniform::from(excluded_x_low_range);
    let excluded_x_high_range = (180f32.next_up())..360f32;
    let excluded_x_high = rand::distributions::Uniform::from(excluded_x_high_range);
    let excluded_y_low_range = -180f32..-90f32;
    let excluded_y_low = rand::distributions::Uniform::from(excluded_y_low_range);
    let excluded_y_high_range = (90f32.next_up())..180f32;
    let excluded_y_high = rand::distributions::Uniform::from(excluded_y_high_range);

    let mut index = HPRTreeWrappingBuilder::new(N_INCLUDED + N_EXCLUDED);

    let bbox_included_lim = BBox {
        minx: -180f32,
        miny: -90f32,
        maxx: 180f32,
        maxy: 90f32,
    };
    let mut bbox_included = BBox::default();
    let mut bbox_all = BBox::default();

    for _ in 0..N_INCLUDED {
        let pt = Point {
            x: included_x.sample(&mut rng),
            y: included_y.sample(&mut rng),
        };
        bbox_included.expand_to_include_point(&pt);
        bbox_all.expand_to_include_point(&pt);
        index.insert(true, pt);
    }
    for _ in 0..N_EXCLUDED {
        let pt = Point {
            x: if rand::random() {
                excluded_x_low.sample(&mut rng)
            } else {
                excluded_x_high.sample(&mut rng)
            },
            y: if rand::random() {
                excluded_y_low.sample(&mut rng)
            } else {
                excluded_y_high.sample(&mut rng)
            },
        };
        bbox_all.expand_to_include_point(&pt);
        index.insert(false, pt);
    }

    let index = index.build();
    assert!(index.len() == N_INCLUDED + N_EXCLUDED);

    let query_included_lim = index.query(&bbox_included_lim);
    assert!(query_included_lim.len() == N_INCLUDED);
    for i in query_included_lim {
        assert!(i);
    }

    let query_included = index.query(&bbox_included);
    assert!(query_included.len() == N_INCLUDED);
    for i in query_included {
        assert!(i);
    }

    let query_all = index.query(&bbox_all);
    assert!(query_all.len() == N_INCLUDED + N_EXCLUDED);

    let bbox_excluded_top = BBox {
        minx: -360f32,
        miny: 90f32.next_up(),
        maxx: 360f32,
        maxy: 180f32,
    };
    let query_top = index.query(&bbox_excluded_top);

    let bbox_excluded_bottom = BBox {
        minx: -360f32,
        miny: -180f32,
        maxx: 360f32,
        maxy: (-90f32).next_down(),
    };
    let query_bottom = index.query(&bbox_excluded_bottom);

    let bbox_excluded_right = BBox {
        minx: 180f32.next_up(),
        miny: -90f32,
        maxx: 360f32,
        maxy: 90f32,
    };
    let query_right = index.query(&bbox_excluded_right);

    let bbox_excluded_left = BBox {
        minx: -360f32,
        miny: -90f32,
        maxx: (-180f32).next_down(),
        maxy: 90f32,
    };
    let query_left = index.query(&bbox_excluded_left);

    assert!(
        query_top.len() + query_left.len() + query_bottom.len() + query_right.len() == N_EXCLUDED
    );

    for i in query_top
        .into_iter()
        .chain(query_left.into_iter())
        .chain(query_bottom.into_iter())
        .chain(query_right.into_iter())
    {
        assert!(!i);
    }
}

#[test]
fn hprtree_end_to_end() {
    let mut index = HPRTreeWrappingBuilder::new(259200);
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
