use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Fields};

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

pub(crate) fn derive_from(input: DeriveInput) -> syn::Result<TokenStream> {
    let (into_ty, into_stmt) = match &input.data {
        Data::Struct(data) => {
            if data.fields.len() != 1 {
                return Err(syn::Error::new_spanned(
                    input,
                    "expected struct with one field",
                ));
            }

            let fields = match &data.fields {
                Fields::Named(fields) => &fields.named,
                Fields::Unnamed(fields) => &fields.unnamed,
                Fields::Unit => {
                    return Err(syn::Error::new_spanned(
                        input,
                        "expected struct with one field",
                    ))
                }
            };

            if fields.len() != 1 {
                return Err(syn::Error::new_spanned(
                    input,
                    "expected struct with one field",
                ));
            }

            // todo: fix bugs, e.g. with named fields
            (&fields[0].ty, quote!(Self(t.into())))
        }
        Data::Enum(_) => {
            todo!("Enums are not supported yet")
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "expected struct with one field",
            ))
        }
    };

    let name = &input.ident;
    let generics = &input.generics;
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let mut new_generics = generics.clone();
    new_generics
        .params
        .push(parse_quote!(__T: ::std::convert::Into<#into_ty>));
    let (impl_generics, _, _) = new_generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::std::convert::From<__T> for #name #ty_generics #where_clause {
            fn from(t: __T) -> Self {
                Self(t.into())
            }
        }
    })
}
