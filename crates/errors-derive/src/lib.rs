//! Derive macro for implementing `serde::Serialize` on thiserror enums.
//!
//! This macro generates serialization that produces JSON in the format:
//! `{"kind":"variantName","message":"The error message from Display"}`

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Variant, parse_macro_input};

/// Derive macro that implements `serde::Serialize` for error enums.
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, Error, SerializableError)]
/// pub enum MyError {
///     #[error("Something went wrong")]
///     SomethingWrong,
///     #[error("IO error: {0}")]
///     Io(#[from] std::io::Error),
/// }
/// ```
///
/// This generates JSON like:
/// - `{"kind":"somethingWrong","message":"Something went wrong"}`
/// - `{"kind":"io","message":"IO error: <details>"}`
///
/// # Custom Error Codes
///
/// Use `#[error_code("CUSTOM_CODE")]` to override the default camelCase conversion:
///
/// ```ignore
/// #[derive(Debug, Error, SerializableError)]
/// pub enum MyError {
///     #[error("File not found")]
///     #[error_code("FILE_NOT_FOUND")]
///     FileNotFound,
/// }
/// ```
#[proc_macro_derive(SerializableError, attributes(error_code))]
pub fn derive_serializable_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let variants = match &input.data {
        syn::Data::Enum(data) => &data.variants,
        _ => {
            return syn::Error::new_spanned(
                input,
                "SerializableError can only be derived for enums",
            )
            .to_compile_error()
            .into();
        }
    };

    let match_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let error_code = get_error_code(variant);

        let pattern = match &variant.fields {
            Fields::Unit => quote! { Self::#variant_name },
            Fields::Unnamed(fields) => {
                let underscores = fields.unnamed.iter().map(|_| quote! { _ });
                quote! { Self::#variant_name(#(#underscores),*) }
            }
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| {
                        let name = f.ident.as_ref().expect("Named field without ident");
                        quote! { #name: _ }
                    })
                    .collect();
                quote! { Self::#variant_name { #(#field_names),* } }
            }
        };

        quote! {
            #pattern => #error_code
        }
    });

    let expanded = quote! {
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeStruct;

                let error_message = self.to_string();
                let error_kind = match self {
                    #(#match_arms),*
                };

                let mut s = serializer.serialize_struct(stringify!(#name), 2)?;
                s.serialize_field("kind", error_kind)?;
                s.serialize_field("message", &error_message)?;
                s.end()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Extract custom error code from `#[error_code("...")]` attribute,
/// or convert variant name to camelCase.
fn get_error_code(variant: &Variant) -> proc_macro2::TokenStream {
    // Check for #[error_code("...")] attribute
    for attr in &variant.attrs {
        if attr.path().is_ident("error_code")
            && let Ok(lit) = attr.parse_args::<syn::LitStr>()
        {
            let code = lit.value();
            return quote! { #code };
        }
    }

    // Default: convert variant name to camelCase
    let variant_name = variant.ident.to_string();
    let kebab_case = variant_name.to_case(Case::Kebab);
    quote! { #kebab_case }
}

