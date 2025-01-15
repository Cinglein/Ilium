use proc_macro::*;
use proc_macro_error::*;

mod queue;
mod state;
mod util;

#[proc_macro_derive(Queue, attributes(ilium))]
#[proc_macro_error]
pub fn queue(item: TokenStream) -> TokenStream {
    queue::derive_queue_impl(item)
}

#[proc_macro_derive(SharedState, attributes(ilium))]
#[proc_macro_error]
pub fn shared(item: TokenStream) -> TokenStream {
    state::derive_state_impl(item, true)
}

#[proc_macro_derive(UserState, attributes(ilium))]
#[proc_macro_error]
pub fn user(item: TokenStream) -> TokenStream {
    state::derive_state_impl(item, false)
}
