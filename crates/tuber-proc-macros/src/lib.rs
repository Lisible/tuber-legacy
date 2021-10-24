use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Tag)]
pub fn derive_tag_trait(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let enum_tag_trait_impl;
    match data {
        Data::Enum(data_enum) => {
            let variants = data_enum.variants.iter().map(|variant| &variant.ident);
            enum_tag_trait_impl = quote! {
                impl IntoTag for #ident {
                    fn into_tag(self) -> Tag {
                        match self {
                            #((#ident::#variants) => Tag::new(stringify!(#variants)), )*
                            _ => panic!("Unknown tag variant")
                        }
                    }
                }
            };
        }

        _ => panic!("Tag is only implemented for enums"),
    }

    enum_tag_trait_impl.into()
}
