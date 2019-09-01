#![recursion_limit = "128"]

extern crate proc_macro;

use quote::quote;

#[proc_macro_derive(EnumNext)]
pub fn proc_macro_derive_enum_next(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    assert!(
        input.generics.params.is_empty(),
        "Can only be applied to enumerations without type parameters."
    );

    let enum_name = &input.ident;

    let enum_data = match input.data {
        syn::Data::Enum(ref data) => data,
        _ => {
            panic!("Can only be applied to enumerations.");
        }
    };

    for variant in enum_data.variants.iter() {
        assert!(
            0 == variant.fields.iter().len(),
            "Cannot construct variant {} because it has has fields.",
            variant.ident
        );
    }

    let next_match = enum_data
        .variants
        .iter()
        .zip(enum_data.variants.iter().skip(1))
        .map(|(a, b)| {
            let ai = &a.ident;
            let bi = &b.ident;
            quote!(#enum_name::#ai => Some(#enum_name::#bi))
        });

    let next_match_last = enum_data.variants.iter().last().map(|a| {
        let ai = &a.ident;
        quote!(#enum_name::#ai => None)
    });

    let wrapping_next_match = enum_data
        .variants
        .iter()
        .zip(enum_data.variants.iter().cycle().skip(1))
        .map(|(a, b)| {
            let ai = &a.ident;
            let bi = &b.ident;
            quote!(#enum_name::#ai => #enum_name::#bi)
        });

    let tokens = quote! {
        impl #enum_name {
            pub fn next(self) -> Option<Self> {
                match self {
                    #(#next_match,)*
                    #next_match_last,
                }
            }

            pub fn wrapping_next(self) -> Self {
                match self {
                    #(#wrapping_next_match,)*
                }
            }

            pub fn wrapping_next_assign(&mut self) {
                *self = self.wrapping_next();
            }
        }
    };

    tokens.into()
}

#[proc_macro_derive(EnumPrev)]
pub fn proc_macro_derive_enum_prev(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    assert!(
        input.generics.params.is_empty(),
        "Can only be applied to enumerations without type parameters."
    );

    let enum_name = &input.ident;

    let enum_data = match input.data {
        syn::Data::Enum(ref data) => data,
        _ => {
            panic!("Can only be applied to enumerations.");
        }
    };

    for variant in enum_data.variants.iter() {
        assert!(
            0 == variant.fields.iter().len(),
            "Cannot construct variant {} because it has has fields.",
            variant.ident
        );
    }

    let prev_match_first = enum_data.variants.iter().next().map(|a| {
        let ai = &a.ident;
        quote!(#enum_name::#ai => None)
    });

    let prev_match = enum_data
        .variants
        .iter()
        .skip(1)
        .zip(enum_data.variants.iter())
        .map(|(a, b)| {
            let ai = &a.ident;
            let bi = &b.ident;
            quote!(#enum_name::#ai => Some(#enum_name::#bi))
        });

    let wrapping_prev_match = enum_data
        .variants
        .iter()
        .zip(enum_data.variants.iter().cycle().skip(1))
        .map(|(b, a)| {
            let ai = &a.ident;
            let bi = &b.ident;
            quote!(#enum_name::#ai => #enum_name::#bi)
        });

    let tokens = quote! {
        impl #enum_name {
            pub fn prev(self) -> Option<Self> {
                match self {
                    #prev_match_first,
                    #(#prev_match,)*
                }
            }

            pub fn wrapping_prev(self) -> Self {
                match self {
                    #(#wrapping_prev_match,)*
                }
            }

            pub fn wrapping_prev_assign(&mut self) {
                *self = self.wrapping_prev();
            }
        }
    };

    tokens.into()
}
