
struct Elem<T> {
    pub elem: T,
    pub next: *mut Elem<T>,
}

pub struct RemoveList<T> {
    data: Vec<Elem<T>>,
    first: *mut Elem<T>,
    capacity: usize,
    size: usize,
    write_index: usize,
    previous: *mut Elem<T>,
}

impl<T> RemoveList<T> {

    pub fn new(capacity: usize) -> Self {
        let mut data = Vec::with_capacity(capacity);
        RemoveList {
            data,
            first: unsafe { &mut data[0] as *mut Elem<T> },
            capacity,
            size: 0,
            write_index: 0,
            previous: unsafe { &mut data[0] as *mut Elem<T> },
        }
    }

    pub fn push(&mut self, elem: T) {
        if self.write_index == self.capacity {
            self.capacity *= 2;
            let new_data = Vec::with_capacity(self.capacity);

            //TODO
        }
        let index = self.write_index;
        self.write_index += 1;
        self.size += 1;
        self.data[index] = Elem {
            elem,
            next: self.first,
        };
        self.previous = self.first;
        self.first = self.data[index];
    }

    pub fn pop(&mut self) -> T {    //TODO
        if self.size == 0 {
            panic!("Pop on empty RemoveList");
        }
        let elem = self.first.elem;
        self.first = self.first.next;
        self.size -= 1;
        elem
    }

    pub fn remove_if(&mut self, predicate: fn(&T) -> bool) {
        if self.size == 0 { return; }

        while predicate(&first.elem) {
            self.first = first.next;
            self.size -= 1;
            if self.size == 0 { return; }
        }

        let mut previous = first;
        let mut current = self.first.next;
        while current != std::mem::uninitialized() {
            if predicate(&current.elem) {
                previous.next = current.next;
                self.size -= 1;
                continue;
            }
            previous = current;
            current = current.next;
        }
    }

    pub fn for_each<F>(&mut self, mut f: F) where F: FnMut(T) {
        if self.size == 0 { return; }
        let mut current = self.first;
        while current != unsafe { std::mem::uninitialized() } {
            f(current.elem);
            current = current.next;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}