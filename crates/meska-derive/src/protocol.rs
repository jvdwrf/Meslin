use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn derive_protocol(input: DeriveInput) -> syn::Result<TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(input, "expected enum"));
    };

    // contains all the fields (single ones) of the enum
    let mut variants = Vec::new();
    for variant in &data.variants {
        let fields = match &variant.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(fields) => &fields.unnamed,
            Fields::Unit => continue,
        };

        if fields.is_empty() {
            continue;
        } else if fields.len() > 1 {
            return Err(syn::Error::new_spanned(
                variant,
                "expected at most one field",
            ));
        }

        variants.push((&variant.ident, &fields[0].ty));
    }

    let variant_types = variants.iter().map(|(_, ty)| ty);
    let variant_idents = variants.iter().map(|(ident, _)| ident);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #(
            impl #impl_generics ::meska::Accept<#variant_types> for #name #ty_generics #where_clause {
                fn from_msg(msg: #variant_types) -> Self {
                    Self::#variant_idents(msg)
                }

                fn try_into_msg(self) -> Result<#variant_types, Self> {
                    match self {
                        Self::#variant_idents(msg) => Ok(msg),
                        _ => Err(self),
                    }
                }
            }
        )*
    })
}
