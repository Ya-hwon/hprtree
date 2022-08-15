mod hprtree;

pub use crate::hprtree::*;

#[cfg(test)]
mod tests {
    use crate::{HPRTree, Point, Envelope};

    #[test]
    fn hprtree_end_to_end() {
        let mut index = HPRTree::<usize>::new(259200);
        let mut x = -180f32;
        for i in 0..(180 * 2 * 2) {
            let mut y = -90f32;
            for j in 0..(90 * 2 * 2) {
                index.insert(i * 1000 + j, &Point{ x, y });
                y += 0.5;
            }
            x += 0.5;
        }
        index.build();
        let list = index.query(
            &Envelope
            { 
                minx: -10f32, 
                miny: -10f32, 
                maxx: 10f32, 
                maxy: 10f32 
            }
        );
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
