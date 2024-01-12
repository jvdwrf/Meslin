#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

#[proc_macro_derive(Protocol, attributes(from))]
pub fn protocol(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = syn::parse_macro_input!(input as syn::DeriveInput);
    quote!().into()
}

#[proc_macro_derive(DynProtocol, attributes(from))]
pub fn dyn_protocol(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = syn::parse_macro_input!(input as syn::DeriveInput);
    quote!().into()
}

#[proc_macro_derive(Message, attributes())]
pub fn message(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // let input = syn::parse_macro_input!(input as syn::DeriveInput);
    quote!().into()
}
