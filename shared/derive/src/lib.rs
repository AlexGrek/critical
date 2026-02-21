use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive macro that generates a `{Name}Brief` struct containing only fields
/// marked with `#[brief]`, plus a `to_brief()` method and `brief_field_names()`.
///
/// # Example
/// ```ignore
/// #[derive(Brief)]
/// struct User {
///     #[brief]
///     id: String,
///     #[brief]
///     name: String,
///     password_hash: String, // excluded from brief
/// }
/// ```
/// Generates:
/// - `UserBrief { id: String, name: String }` with Serialize, Deserialize, Debug, Clone
/// - `User::to_brief(&self) -> UserBrief`
/// - `User::brief_field_names() -> &'static [&'static str]`
#[proc_macro_derive(Brief, attributes(brief))]
pub fn derive_brief(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_brief(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_brief(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let brief_name = format_ident!("{}Brief", name);
    let vis = &input.vis;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    name,
                    "Brief can only be derived for structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "Brief can only be derived for structs",
            ))
        }
    };

    let brief_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("brief")))
        .collect();

    if brief_fields.is_empty() {
        return Err(syn::Error::new_spanned(
            name,
            "At least one field must be marked with #[brief]",
        ));
    }

    // Generate brief struct fields, carrying over all non-brief attributes (e.g. serde)
    let brief_struct_fields = brief_fields.iter().map(|f| {
        let field_name = &f.ident;
        let ty = &f.ty;
        let attrs: Vec<_> = f
            .attrs
            .iter()
            .filter(|a| !a.path().is_ident("brief"))
            .collect();
        quote! {
            #(#attrs)*
            pub #field_name: #ty
        }
    });

    // Generate to_brief() field assignments
    let brief_assignments = brief_fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! { #field_name: self.#field_name.clone() }
    });

    // Generate field name strings for JSON filtering.
    // Use the Rust field name — after to_external(), JSON keys match field names.
    let field_name_strs = brief_fields.iter().map(|f| {
        let name_str = f.ident.as_ref().unwrap().to_string();
        quote! { #name_str }
    });

    Ok(quote! {
        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
        #vis struct #brief_name {
            #(#brief_struct_fields,)*
        }

        impl #name {
            /// Convert to the brief representation (only fields marked `#[brief]`).
            pub fn to_brief(&self) -> #brief_name {
                #brief_name {
                    #(#brief_assignments,)*
                }
            }

            /// Returns the field names included in the brief representation.
            /// These match the JSON keys after `to_external()` transformation.
            pub fn brief_field_names() -> &'static [&'static str] {
                &[#(#field_name_strs,)*]
            }
        }
    })
}
