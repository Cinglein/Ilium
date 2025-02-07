use session::msg::Message;

pub struct Handle<T: Message> {
    v: Vec<T>,
}

impl<T: Message> Handle<T> {
    pub fn new(url: &str) -> Self {
        //leptos_use::use_websocket::<T, T, codee::binary::RkyvCodec>(url);
        Self { v: Vec::new() }
    }
    pub fn send(&self) {}
    pub fn connect(&mut self) {}
}
