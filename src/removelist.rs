struct Elem<T> {
    pub next: usize,
    pub elem: T,
}

pub struct RemoveList<T> {
    data: Vec<Elem<T>>,
    first: usize,
    size: usize,
    last: usize,
}

impl<T> RemoveList<T> {

    pub fn new(capacity: usize) -> Self {
        RemoveList {
            data: Vec::with_capacity(capacity),
            first: 0,
            size: 0,
            last: 0,
        }
    }

    pub fn push(&mut self, elem: T) {

        self.data.push(Elem {
            elem,
            next: 0
        });
        
        self.data[self.last].next = self.data.len() - 1;
        self.last = self.data.len() - 1;
        self.size += 1;
    }

    /*pub fn pop(&mut self) -> T {    //TODO
        if self.size == 0 {
            panic!("Pop on empty RemoveList");
        }
        let elem = self.first.elem;
        self.first = self.first.next;
        self.size -= 1;
        elem
    }*/

    pub fn remove_if(&mut self, predicate: fn(&T) -> bool) {
        if self.size == 0 { return; }

        while predicate(&self.data[self.first].elem) {
            self.first = self.data[self.first].next;
            self.size -= 1;
            if self.size == 0 { return; }
        }

        let mut curr = self.data[self.first].next;
        let mut prev = self.first;
        
        while curr != 0 {
            if predicate(&self.data[curr].elem) {
                self.data[prev].next = self.data[curr].next;
                self.size -= 1;
            } else {
                prev = curr;
            }
            curr = self.data[curr].next;
        }
    }

    pub fn for_each<F>(&mut self, f: F) where F: Fn(&T) {
        if self.size == 0 { return; }
        let mut current = self.first;

        f(&self.data[current].elem);
        current = self.data[current].next;

        while current != 0 {
            f(&self.data[current].elem);
            current = self.data[current].next;
        }
    }

    pub fn for_each_mut<F>(&mut self, mut f: F) where F: FnMut(&T) {
        if self.size == 0 { return; }
        let mut current = self.first;

        while current != 0 {
            f(&self.data[current].elem);
            current = self.data[current].next;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
}