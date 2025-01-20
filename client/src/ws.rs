use session::msg::Message;

pub struct Handle<T: Message> {
    v: Vec<T>,
}

impl<T: Message> Handle<T> {
    pub fn new(url: &str) -> Self {
        Self { v: Vec::new() }
    }
}
