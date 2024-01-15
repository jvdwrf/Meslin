#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod dyn_from_into;
mod message;
// mod protocol;

// #[proc_macro_derive(Protocol, attributes(meslin))]
// pub fn derive_protocol(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let input = parse_macro_input!(input as syn::DeriveInput);
//     protocol::derive_protocol(input)
//         .unwrap_or_else(|e| e.to_compile_error())
//         .into()
// }

#[proc_macro_derive(DynFromInto, attributes(meslin))]
pub fn derive_dyn_from_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    dyn_from_into::derive_dyn_from_into(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(Message, attributes(meslin))]
pub fn derive_message(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    message::derive_message(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
