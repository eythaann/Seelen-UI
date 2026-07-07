use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident};

#[proc_macro_attribute]
pub fn model(attr: TokenStream, item: TokenStream) -> TokenStream {
    let has_default = if attr.is_empty() {
        false
    } else {
        let ident = parse_macro_input!(attr as Ident);

        match ident.to_string().as_str() {
            "default" => true,
            _ => {
                return syn::Error::new_spanned(ident, "expected `default`")
                    .to_compile_error()
                    .into();
            }
        }
    };

    let input = parse_macro_input!(item as DeriveInput);

    let ident = input.ident;
    let vis = input.vis;
    let generics = input.generics;

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => {
            return syn::Error::new_spanned(ident, "#[model] can only be used on structs")
                .to_compile_error()
                .into();
        }
    };

    let body = match fields {
        Fields::Named(fields) => quote! {
            struct #ident #generics #fields
        },
        Fields::Unnamed(fields) => quote! {
            struct #ident #generics #fields;
        },
        Fields::Unit => quote! {
            struct #ident #generics;
        },
    };

    let derives = if has_default {
        quote! {
            #[derive(
                Debug,
                Default,
                Clone,
                serde::Serialize,
                serde::Deserialize,
                schemars::JsonSchema,
            )]
            #[serde(default, rename_all = "camelCase")]
        }
    } else {
        quote! {
            #[derive(
                Debug,
                Clone,
                serde::Serialize,
                serde::Deserialize,
                schemars::JsonSchema,
            )]
            #[serde(rename_all = "camelCase")]
        }
    };

    quote! {
        #derives
        #[cfg_attr(feature = "salvo", derive(salvo::oapi::ToSchema))]
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS), ts(export))]
        #vis #body
    }
    .into()
}

#[proc_macro_attribute]
pub fn value_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let ident = input.ident;
    let vis = input.vis;
    let generics = input.generics;

    let variants = match input.data {
        Data::Enum(data) => {
            for variant in &data.variants {
                if !matches!(variant.fields, Fields::Unit) {
                    return syn::Error::new_spanned(
                        &variant.ident,
                        "#[pure_enum] only supports fieldless enum variants",
                    )
                    .to_compile_error()
                    .into();
                }
            }

            data.variants
        }
        _ => {
            return syn::Error::new_spanned(ident, "#[pure_enum] can only be used on enums")
                .to_compile_error()
                .into();
        }
    };

    quote! {
        #[derive(
            Debug,
            Clone,
            Copy,
            Hash,
            PartialEq,
            Eq,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
        )]
        #[cfg_attr(feature = "salvo", derive(salvo::oapi::ToSchema))]
        #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS), ts(export, repr(enum = name)))]
        #vis enum #ident #generics {
            #variants
        }
    }
    .into()
}
