pub struct Worker {
    id: usize,
}

impl Worker {
    pub fn new(id: usize) -> Self {
        println!("Creating worker {}", id);
        Self { id }
    }

    pub fn call(&self) {
        println!("Test calling worker {}", self.id);
    }
}
