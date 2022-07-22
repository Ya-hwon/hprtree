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
        let first: *mut Elem<T> = &mut data[0] as *mut Elem<T>;
        let previous: *mut Elem<T> = &mut data[0] as *mut Elem<T>;
        RemoveList {
            data,
            first,
            capacity,
            size: 0,
            write_index: 0,
            previous,
        }
    }

    pub fn push(&mut self, elem: T) {
        if self.write_index == self.capacity {
            self.capacity *= 2;
            let mut new_data = Vec::with_capacity(self.capacity);
            let mut index = 0;
        
            let mut current = self.first;
            while current != std::ptr::null_mut() {
                unsafe {
                    new_data[index] = Elem {
                        elem: (*current).elem,  // ok, so this isnt gonna work huh...
                        next: &mut new_data[index+1] as *mut Elem<T>,
                    };
                
                    current = (*current).next;
                    index += 1;
                }
            }
        
            self.previous = &mut new_data[index-1] as *mut Elem<T>;
            self.data = new_data;
            self.first = &mut self.data[0] as *mut Elem<T>;
        }
        unsafe {
            (*self.previous).next = &mut self.data[self.write_index] as *mut Elem<T>;
        }
        self.data[self.write_index].next = std::ptr::null_mut();
        self.data[self.write_index].elem = elem;
        self.write_index += 1;
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

        unsafe {
        while predicate(&(*self.first).elem) {
            self.first = (*self.first).next;
            self.size -= 1;
            if self.size == 0 { return; }
        }

        let mut previous = self.first;
        let mut current = (*self.first).next;
        while current != std::ptr::null_mut() {
            if predicate(&(*current).elem) {
                (*previous).next = (*current).next;
                self.size -= 1;
                continue;
            }
            previous = current;
            current = (*current).next;
        }
    }}

    pub fn for_each<F>(&mut self, mut f: F) where F: FnMut(&T) {
        if self.size == 0 { return; }
        let mut current = self.first;
        while current != std::ptr::null_mut() {
            unsafe{
            f(&(*current).elem);
            current = (*current).next;
        }}
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