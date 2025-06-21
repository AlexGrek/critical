
extern crate proc_macro;

mod macros;

use proc_macro::TokenStream;
use syn::parse_macro_input; // Import parse_macro_input! here

/// Derive macro for `GitopsResourceRoot`.
///
/// This macro implements the `GitopsResourceRoot` trait for the annotated struct.
/// It requires a `key` attribute to specify the field that acts as the unique identifier
/// for the resource (e.g., `#[gitops(key = "name")]`).
/// An optional `api_version` attribute can be used to set the `apiVersion` field
/// in the generated serializable struct (default is "v1.0").
///
/// It generates:
/// - A `Clone` implementation for the original struct.
/// - A `MyStructGitopsSerializable` struct with `Debug`, `Serialize`, `Deserialize` derives,
///   and additional `kind` (struct name) and `apiVersion` fields.
/// - `as_serializable()`, `into_serializable()`, `From<MyStructGitopsSerializable>`
///   implementations for conversion.
/// - A `MyStructGitopsUpdate` struct with `Debug`, `Serialize`, `Deserialize` derives,
///   where most fields are `Option<T>` for partial updates.
/// - A `with_updates_from()` method for merging updates into the resource.
///
/// Field attributes:
/// - `#[gitops(skip_on_update)]`: This field will not be updated by `with_updates_from()`.
/// - `#[gitops(required_in_update)]`: This field in the `_Update` struct will not be wrapped
///   in `Option`, meaning it must always be present in update payloads.
#[proc_macro_derive(GitopsResourceRoot, attributes(gitops))]
pub fn gitops_resource_root_derive(input: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a DeriveInput once here.
    let ast = parse_macro_input!(input as syn::DeriveInput);
    macros::gitops_resource_root_derive_impl(ast)
        .unwrap_or_else(|err| err.to_compile_error().into()).into() // Convert proc_macro2::TokenStream to proc_macro::TokenStream
}

/// Derive macro for `GitopsResourcePart`.
///
/// This macro implements the `GitopsResourcePart` trait for the annotated struct.
/// It automatically adds `Clone`, `Debug`, `Serialize`, and `Deserialize` derives
/// to the original struct.
///
/// It also generates a `MyStructGitopsUpdate` struct for nested parts to enable
/// recursive merging within `with_updates_from` logic.
///
/// Field attributes for update struct:
/// - `#[gitops(required_in_update)]`: This field in the part's `_Update` struct
///   will not be wrapped in `Option`.
#[proc_macro_derive(GitopsResourcePart, attributes(gitops))]
pub fn gitops_resource_part_derive(input: TokenStream) -> TokenStream {
    // Parse the input TokenStream into a DeriveInput once here.
    let ast = parse_macro_input!(input as syn::DeriveInput);
    macros::gitops_resource_part_derive_impl(ast)
        .unwrap_or_else(|err| err.to_compile_error().into()).into() // Convert proc_macro2::TokenStream to proc_macro::TokenStream
}

#[proc_macro_derive(GitopsEnum)]
pub fn gitops_enum_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    macros::gitops_enum_derive_impl(ast)
        .unwrap_or_else(|err| err.to_compile_error().into()).into()
}
