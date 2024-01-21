#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod from_into_boxed;
mod message;

#[proc_macro_derive(DynProtocol, attributes())]
pub fn derive_from_into_boxed(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    from_into_boxed::derive(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(Message, attributes())]
pub fn derive_message(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    message::derive(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
