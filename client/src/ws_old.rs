use leptos::{leptos_dom::helpers::TimeoutHandle, logging, prelude::*};
use session::{
    info::StateInfo,
    msg::{Message, Msg},
    queue::Queue,
};
use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, Event, MessageEvent, WebSocket};

const RECONNECT_INTERVAL: u64 = 3000;
const RECONNECT_LIMIT: u64 = 3;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub enum Ready {
    Connecting,
    Open,
    Closed,
}

#[derive(Clone, Debug)]
pub struct Handle<I: 'static + Message> {
    ready: Signal<Ready>,
    connect: ArcStoredValue<Option<std::rc::Rc<dyn Fn()>>>,
    reconnect: StoredValue<u64>,
    pub ws: ArcStoredValue<Option<WebSocket>>,
    pub set_info: WriteSignal<StateInfo<I>>,
}

impl<I: 'static + Message> Handle<I> {
    pub fn open(&self) {
        if self.ready.get_untracked() == Ready::Closed {
            self.reconnect.set_value(0);
            if let Some(connect) = self.connect.get_value() {
                connect();
            } else {
                logging::log!("no connect ref");
            }
        }
    }
    pub fn close(&self) {
        self.reconnect.set_value(RECONNECT_LIMIT);
        if let Some(ws) = self.ws.get_value() {
            let _ = ws.close();
        }
    }
    fn send_raw(&self, data: &[u8]) {
        self.open();
        if self.ready.get_untracked() == Ready::Open {
            if let Some(ws) = self.ws.get_value() {
                let _ = ws.send_with_u8_array(data);
                logging::log!("sent {data:?}");
            } else {
                logging::log!("no websocket");
            }
        }
    }
    pub fn send<Q: Queue>(&self, msg: Msg<Q>) {
        if let Ok(data) = postcard::to_allocvec::<Msg<Q>>(&msg) {
            self.send_raw(&data);
        }
    }
}

pub fn ws_handle<I: 'static + Message>(
    url: &'static str,
    set_info: WriteSignal<StateInfo<I>>,
) -> Handle<I> {
    let url = normalize_url(url);
    let (ready, set_ready) = signal(Ready::Closed);

    let ws_ref: ArcStoredValue<Option<WebSocket>> = ArcStoredValue::new(None);
    let reconnect_timer_ref: ArcStoredValue<Option<TimeoutHandle>> = ArcStoredValue::new(None);
    let reconnect_times_ref: StoredValue<u64> = StoredValue::new(0);
    let unmounted = std::rc::Rc::new(std::cell::Cell::new(false));
    let connect_ref: ArcStoredValue<Option<std::rc::Rc<dyn Fn()>>> = ArcStoredValue::new(None);

    let reconnect_ref: ArcStoredValue<Option<std::rc::Rc<dyn Fn()>>> = ArcStoredValue::new(None);
    reconnect_ref.set_value({
        let ws = ws_ref.get_value();
        Some(std::rc::Rc::new(move || {
            let ws_not_open = ws
                .clone()
                .is_some_and(|ws: WebSocket| ws.ready_state() != WebSocket::OPEN);
            if ws_not_open && reconnect_times_ref.get_value() < RECONNECT_LIMIT {
                set_ready(Ready::Connecting);
                reconnect_timer_ref.set_value(
                    set_timeout_with_handle(
                        move || {
                            if let Some(connect) = connect_ref.get_value() {
                                connect();
                                reconnect_times_ref.update_value(|current| *current += 1);
                            }
                        },
                        std::time::Duration::from_millis(RECONNECT_INTERVAL),
                    )
                    .ok(),
                );
            }
        }))
    });
    connect_ref.set_value({
        let ws = ws_ref.get_value();
        let unmounted = std::rc::Rc::clone(&unmounted);
        Some(std::rc::Rc::new(move || {
            reconnect_timer_ref.set_value(None);
            if let Some(websocket) = &ws {
                let _ = websocket.close();
            }
            let websocket = WebSocket::new(&url).unwrap_throw();
            websocket.set_binary_type(BinaryType::Arraybuffer);
            set_ready.set(Ready::Connecting);
            {
                let unmounted = std::rc::Rc::clone(&unmounted);
                let onopen = Closure::wrap(Box::new(move |_e: Event| {
                    if !unmounted.get() {
                        set_ready.set(Ready::Open);
                    }
                }) as Box<dyn FnMut(Event)>);
                websocket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                onopen.forget();
            }
            {
                let unmounted = std::rc::Rc::clone(&unmounted);
                let onmessage = Closure::wrap(Box::new(move |msg: MessageEvent| {
                    if !unmounted.get() {
                        if let Ok(buf) = msg.data().dyn_into::<js_sys::ArrayBuffer>() {
                            let bytes: Vec<u8> = js_sys::Uint8Array::new(&buf).to_vec();
                            if let Ok(info) = postcard::from_bytes::<StateInfo<I>>(&bytes) {
                                set_info.try_set(info);
                            }
                        }
                    }
                }) as Box<dyn FnMut(MessageEvent)>);
                websocket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                onmessage.forget();
            }
            {
                let unmounted = std::rc::Rc::clone(&unmounted);
                let onclose = Closure::wrap(Box::new(move |_e: Event| {
                    if !unmounted.get() {
                        if let Some(reconnect) = &reconnect_ref.get_value() {
                            reconnect();
                        }
                        set_info(StateInfo::Closed);
                        set_ready(Ready::Closed);
                    }
                }) as Box<dyn FnMut(Event)>);
                websocket.set_onerror(Some(onclose.as_ref().unchecked_ref()));
                websocket.set_onclose(Some(onclose.as_ref().unchecked_ref()));
                onclose.forget();
            }
            ws_ref.set_value(Some(websocket));
        }))
    });
    let close = move || {
        reconnect_times_ref.set_value(RECONNECT_LIMIT);
        if let Some(websocket) = ws_ref.get_value() {
            let _ = websocket.close();
        }
    };
    Effect::new(move |_| {
        if let Some(connect) = connect_ref.get_value() {
            connect();
        }
    });
    on_cleanup(move || {
        //unmounted.set(true);
        //close();
    });
    Handle {
        ready: ready.into(),
        connect: connect_ref,
        reconnect: reconnect_times_ref,
        ws: ws_ref,
        set_info,
    }
}

fn normalize_url(url: &str) -> String {
    if url.starts_with("ws://") || url.starts_with("wss://") {
        url.to_string()
    } else if url.starts_with("//") {
        format!(
            "{}{}",
            window()
                .location()
                .protocol()
                .expect("Protocol not found")
                .replace("http", "ws"),
            url,
        )
    } else if url.starts_with('/') {
        format!(
            "{}//{}{}",
            window()
                .location()
                .protocol()
                .expect("Protocol not found")
                .replace("http", "ws"),
            window().location().host().expect("Host not found"),
            url
        )
    } else {
        let mut path = window().location().pathname().expect("Pathname not found");
        if !path.ends_with('/') {
            path.push('/')
        }
        format!(
            "{}//{}{}{}",
            window()
                .location()
                .protocol()
                .expect("Protocol not found")
                .replace("http", "ws"),
            window().location().host().expect("Host not found"),
            path,
            url
        )
    }
}
