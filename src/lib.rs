mod hprtree;
mod removelist;

pub use crate::hprtree::*;
pub use crate::removelist::*;

#[cfg(test)]
mod tests {
    use crate::{HPRTree, Point, Envelope, RemoveList};

    #[test]
    fn general_removelist() {
        let mut removelist: RemoveList<usize> = RemoveList::<usize>::new(3);
        removelist.remove_if(|_elem| {
            true
        });
        assert!(removelist.size() == 0);
        removelist.for_each(|_elem| {
            assert!(false);
        });
        //assert!(removelist.to_vec().len() == 0);
        removelist.remove_if(|_elem| {
            assert!(false);
            false
        });
        removelist.push(1);
        removelist.push(2);
        removelist.push(3);
        assert!(removelist.size() == 3);
        assert!(removelist.capacity() >= 3);
    }

    #[test]
    fn content_removelist() {
        let numentries = 1_000;

        let mut removelist: RemoveList<usize> = RemoveList::<usize>::new(numentries);
        assert!(removelist.size() == 0);
        assert!(removelist.capacity() >= numentries);
        for i in 0..numentries {
            removelist.push(i);
        }
        assert!(removelist.size() == numentries);
        assert!(removelist.capacity() >= numentries);
        let mut i: usize = 0;
        removelist.for_each(|elem| {
            assert!(*elem == i);
            i += 1;
        });
        assert!(removelist.size() == numentries);
        assert!(removelist.capacity() >= numentries);
        removelist.remove_if(|elem|{
            *elem % 2 == 0
        });
        assert!(removelist.size() == numentries / 2);
        assert!(removelist.capacity() >= numentries / 2);
        i = 0;
        removelist.for_each(|elem| {
            assert!(*elem == i);
            i += 2;
        });
        removelist.remove_if(|elem|{
            *elem % 2 != 0
        });
        assert!(removelist.size() == 0);
    }

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
