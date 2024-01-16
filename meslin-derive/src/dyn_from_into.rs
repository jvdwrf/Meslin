use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Enum(data) = input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "DynFromInto can only be derived for enums",
        ));
    };

    let variant_names = data
        .variants
        .iter()
        .map(|variant| &variant.ident)
        .collect::<Vec<_>>();
    let variant_types = data
        .variants
        .iter()
        .map(|variant| {
            let fields = match &variant.fields {
                syn::Fields::Unnamed(fields) => fields.unnamed.iter().collect::<Vec<_>>(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        variant,
                        "DynFromInto can only be derived for enums with unnamed fields",
                    ))
                }
            };
            if fields.len() != 1 {
                return Err(syn::Error::new_spanned(
                    variant,
                    "DynFromInto can only be derived for enums with exactly one field",
                ));
            }
            Ok(&fields[0].ty)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let len = variant_names.len();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics meslin::AcceptsList for #name #ty_generics #where_clause {
            fn accepts_list() -> &'static [std::any::TypeId] {
                static LOCK: std::sync::OnceLock<[std::any::TypeId; #len]> = std::sync::OnceLock::new();
                LOCK.get_or_init(|| {
                    [
                        #(std::any::TypeId::of::<#variant_types>()),*,
                    ]
                })
            }
        }

        #[automatically_derived]
        impl #impl_generics meslin::DynFromInto for #name #ty_generics #where_clause {
            fn try_from_boxed_msg<_W: 'static>(
                msg: meslin::BoxedMsg<_W>,
            ) -> Result<(Self, _W), meslin::BoxedMsg<_W>> {
                #(
                    let msg = match msg.downcast::<#variant_types>() {
                        Ok((msg, with)) => return Ok((Self::#variant_names(msg), with)),
                        Err(msg) => msg,
                    };
                )*
                Err(msg)
            }

            fn into_boxed_msg<_W: Send + 'static>(self, with: _W) -> meslin::BoxedMsg<_W> {
                match self {
                    #(
                        Self::#variant_names(msg) => meslin::BoxedMsg::new(msg, with),
                    )*
                }
            }
        }

        #(
            #[automatically_derived]
            impl #impl_generics meslin::Accepts<#variant_types> for #name #ty_generics #where_clause {}
        )*
    })
}
