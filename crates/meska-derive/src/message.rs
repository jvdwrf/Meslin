use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn derive_message(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::meska::Message for #name #ty_generics #where_clause {
            type Input = Self;
            type Output = ();

            fn create(from: Self::Input) -> (Self, Self::Output) {
                (from, ())
            }

            fn cancel(self, _: Self::Output) -> Self::Input {
                self
            }
        }
    })
}
