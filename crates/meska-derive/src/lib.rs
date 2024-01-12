#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod protocol;
mod message;

#[proc_macro_derive(Protocol, attributes())]
pub fn derive_protocol(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    protocol::derive_protocol(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(DynProtocol, attributes())]
pub fn derive_dyn_protocol(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = syn::parse_macro_input!(input as syn::DeriveInput);
    quote!().into()
}

#[proc_macro_derive(Message, attributes())]
pub fn derive_message(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    message::derive_message(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

