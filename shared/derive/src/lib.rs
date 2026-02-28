use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, ItemStruct, Meta, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

// ---------------------------------------------------------------------------
// #[crit_resource(...)] attribute macro
// ---------------------------------------------------------------------------

/// Configuration parsed from `#[crit_resource(collection = "...", prefix = "...", no_acl)]`.
struct CritResourceArgs {
    collection: String,
    prefix: String,
    no_acl: bool,
}

impl Parse for CritResourceArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut collection = None;
        let mut prefix = None;
        let mut no_acl = false;

        let metas = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        for meta in metas {
            match &meta {
                Meta::NameValue(nv) if nv.path.is_ident("collection") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        collection = Some(s.value());
                    }
                }
                Meta::NameValue(nv) if nv.path.is_ident("prefix") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        prefix = Some(s.value());
                    }
                }
                Meta::Path(p) if p.is_ident("no_acl") => {
                    no_acl = true;
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &meta,
                        "unexpected argument; expected `collection = \"...\"`, `prefix = \"...\"`, or `no_acl`",
                    ));
                }
            }
        }

        Ok(CritResourceArgs {
            collection: collection.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing `collection = \"...\"`")
            })?,
            prefix: prefix.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing `prefix = \"...\"`")
            })?,
            no_acl,
        })
    }
}

/// Attribute macro that wraps a struct to inject standard resource fields and
/// generate companion code (Brief struct, hash computation, static metadata).
///
/// # Usage
/// ```ignore
/// #[crit_resource(collection = "users", prefix = "u_", no_acl)]
/// pub struct User {
///     pub password_hash: String,
///     #[brief]
///     pub personal: PersonalInfo,
/// }
/// ```
///
/// ## Injected fields (at the top of the struct)
/// - `id: PrincipalId` (with `#[serde(rename = "_key")]`)
/// - `labels: Labels` (with `#[serde(default)]`) — queryable key-value pairs
/// - `annotations: Labels` (with `#[serde(default)]`) — freeform key-value pairs
/// - `acl: AccessControlStore` (unless `no_acl`, with `#[serde(default)]`)
/// - `state: ResourceState` (with `#[serde(default)]`) — server-managed audit timestamps
/// - `deletion: Option<DeletionInfo>` (with `#[serde(default, skip_serializing_if = "Option::is_none")]`)
/// - `hash_code: String` (with `#[serde(default)]`)
///
/// ## Generated code
/// - `{Name}Brief` struct (from `#[brief]` fields, including injected `id`, `labels`)
/// - `impl {Name}` with: `to_brief()`, `brief_field_names()`, `compute_hash()`,
///   `with_computed_hash()`, `collection_name()`, `id_prefix()`
#[proc_macro_attribute]
pub fn crit_resource(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as CritResourceArgs);
    let input = parse_macro_input!(item as ItemStruct);

    match impl_crit_resource(&args, &input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_crit_resource(args: &CritResourceArgs, input: &ItemStruct) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let vis = &input.vis;
    let brief_name = format_ident!("{}Brief", name);
    let collection = &args.collection;
    let prefix = &args.prefix;

    // Extract user-defined fields
    let user_fields = match &input.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "crit_resource can only be applied to structs with named fields",
            ))
        }
    };

    // Determine which user fields are marked #[brief]
    let user_brief_fields: Vec<_> = user_fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("brief")))
        .collect();

    // Collect user-defined field definitions, stripping #[brief] attributes
    // (they're only meaningful to this macro, not to rustc)
    let user_field_defs = user_fields.iter().map(|f| {
        let field_name = &f.ident;
        let ty = &f.ty;
        let vis = &f.vis;
        let attrs: Vec<_> = f
            .attrs
            .iter()
            .filter(|a| !a.path().is_ident("brief"))
            .collect();
        quote! {
            #(#attrs)*
            #vis #field_name: #ty
        }
    });

    // Build the ACL field injection (optional)
    let acl_field = if !args.no_acl {
        quote! {
            #[serde(default)]
            pub acl: crate::util_models::AccessControlStore,
        }
    } else {
        quote! {}
    };

    // Build the full struct with injected fields
    // Preserve user-provided attributes (except #[crit_resource] which is already consumed)
    let user_attrs: Vec<_> = input.attrs.iter().collect();

    let struct_def = quote! {
        #(#user_attrs)*
        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Default)]
        #vis struct #name {
            // ---- injected by crit_resource ----
            #[serde(rename = "_key")]
            pub id: crate::util_models::PrincipalId,
            #[serde(default)]
            pub labels: crate::util_models::Labels,
            #[serde(default)]
            pub annotations: crate::util_models::Labels,
            #acl_field
            #[serde(default)]
            pub state: crate::util_models::ResourceState,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub deletion: Option<crate::util_models::DeletionInfo>,
            #[serde(default)]
            pub hash_code: String,
            // ---- user-defined fields ----
            #(#user_field_defs,)*
        }
    };

    // --- Brief struct generation ---
    // Brief always includes injected `id` and `meta`, plus user fields marked #[brief]

    // Generate brief fields for user-defined brief fields
    let user_brief_struct_fields = user_brief_fields.iter().map(|f| {
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

    // Brief field assignments for to_brief()
    let user_brief_assignments = user_brief_fields.iter().map(|f| {
        let field_name = &f.ident;
        quote! { #field_name: self.#field_name.clone() }
    });

    // Brief field name strings (for JSON filtering)
    let user_brief_name_strs = user_brief_fields.iter().map(|f| {
        let name_str = f.ident.as_ref().unwrap().to_string();
        quote! { #name_str }
    });

    let brief_def = quote! {
        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
        #vis struct #brief_name {
            // injected brief fields
            pub id: crate::util_models::PrincipalId,
            #[serde(default)]
            pub labels: crate::util_models::Labels,
            // user-defined brief fields
            #(#user_brief_struct_fields,)*
        }
    };

    let impl_def = quote! {
        impl #name {
            /// Convert to the brief representation.
            pub fn to_brief(&self) -> #brief_name {
                #brief_name {
                    id: self.id.clone(),
                    labels: self.labels.clone(),
                    #(#user_brief_assignments,)*
                }
            }

            /// Returns the field names included in the brief representation.
            pub fn brief_field_names() -> &'static [&'static str] {
                &["id", "labels", #(#user_brief_name_strs,)*]
            }

            /// ArangoDB collection name for this resource kind.
            pub fn collection_name() -> &'static str {
                #collection
            }

            /// ID prefix for this resource kind (e.g. "u_", "g_").
            pub fn id_prefix() -> &'static str {
                #prefix
            }

            /// Compute FNV-1a hash of desired-state fields (everything except
            /// hash_code, deletion, state). Returns 16-char hex string.
            pub fn compute_hash(&self) -> String {
                // Serialize to JSON, then remove non-desired-state fields
                let mut val = serde_json::to_value(self).unwrap_or_default();
                if let Some(obj) = val.as_object_mut() {
                    obj.remove("hash_code");
                    obj.remove("deletion");
                    obj.remove("state"); // server-managed audit, not desired state
                    // _id and _rev are ArangoDB internals, not desired state
                    obj.remove("_id");
                    obj.remove("_rev");
                }
                let canonical = serde_json::to_string(&val).unwrap_or_default();

                // FNV-1a 64-bit
                let mut hash: u64 = 0xcbf29ce484222325;
                for byte in canonical.as_bytes() {
                    hash ^= *byte as u64;
                    hash = hash.wrapping_mul(0x100000001b3);
                }
                format!("{:016x}", hash)
            }

            /// Set hash_code to the computed hash of current desired state.
            pub fn with_computed_hash(&mut self) {
                // TODO: use it in controllers when writing to DB!
                // TODO: use it in write conflict checks!
                // TODO: send it to clients, implement it in CLI and web UI
                // TODO: use it in history records!
                self.hash_code = self.compute_hash();
            }
        }
    };

    Ok(quote! {
        #struct_def
        #brief_def
        #impl_def
    })
}

// ---------------------------------------------------------------------------
// #[derive(Brief)] — kept for backward compatibility during migration
// ---------------------------------------------------------------------------

/// Derive macro that generates a `{Name}Brief` struct containing only fields
/// marked with `#[brief]`, plus a `to_brief()` method and `brief_field_names()`.
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
