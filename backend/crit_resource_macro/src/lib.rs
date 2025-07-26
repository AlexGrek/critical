extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemStruct, Attribute, Fields, Field};

#[proc_macro]
pub fn custom_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let struct_name = &input.ident;
    let vis = &input.vis;

    // Add #[serde(default)] to each field
    let mut new_fields = Vec::new();
    for mut field in input.fields.iter().cloned() {
        let serde_attr: Attribute = syn::parse_quote!(#[serde(default)]);
        field.attrs.push(serde_attr);
        new_fields.push(field);
    }

    let fields = match &input.fields {
        Fields::Named(_) => quote! {
            {
                #(#new_fields),*
            }
        },
        _ => unimplemented!("Only named structs supported"),
    };

    // Generate the final struct
    let expanded = quote! {
        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        #vis struct #struct_name #fields

        impl MyTrait for #struct_name {
            // implement trait methods here
        }
    };

    expanded.into()
}
