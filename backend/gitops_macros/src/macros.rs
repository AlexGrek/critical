use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream}; // Import Parse and ParseStream
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Field, GenericArgument, Lit, LitStr, Meta, MetaNameValue,
    PathArguments, Token, Type, TypePath,
}; // Used for naming conventions

/// Helper struct to parse `#[gitops(...)]` attributes.
/// Syn's `parse_args` expects a type that implements `Parse`.
struct GitopsAttributeArgs {
    args: Punctuated<Meta, Token![,]>,
}

impl Parse for GitopsAttributeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(GitopsAttributeArgs {
            args: Punctuated::parse_terminated(input)?,
        })
    }
}

/// Helper function to validate if a given type is compatible with GitOps resource fields.
/// Allowed types are:
/// - Primitive types (u8, i32, bool, f64, etc.)
/// - `String`
/// - `Option<T>` where T is a valid GitOps type.
/// - `Vec<T>` where T is a valid GitOps type.
/// - `HashMap<String, V>` or `BTreeMap<String, V>` where V is a valid GitOps type.
/// - Any other struct/enum that is annotated with `#[derive(GitopsResourcePart)]`.
fn validate_gitops_field_type(ty: &Type) -> Result<(), syn::Error> {
    match ty {
        Type::Path(type_path) => {
            let segment = type_path.path.segments.last().ok_or_else(|| {
                syn::Error::new_spanned(ty, "Type path has no segments.")
            })?;
            let ident_str = segment.ident.to_string();

            // Whitelisted primitive types and String
            if ["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64", "bool", "char", "String"].contains(&ident_str.as_str()) {
                Ok(())
            } else if ident_str == "Option" {
                // Recursively validate inner type of Option
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        validate_gitops_field_type(inner_ty)
                    } else {
                        Err(syn::Error::new_spanned(ty, "Option must have a generic argument (e.g., Option<T>)."))
                    }
                } else {
                    Err(syn::Error::new_spanned(ty, "Option must have angle-bracketed generic arguments."))
                }
            } else if ident_str == "Vec" {
                // Recursively validate inner type of Vec
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        validate_gitops_field_type(inner_ty)
                    } else {
                        Err(syn::Error::new_spanned(ty, "Vec must have a generic argument (e.g., Vec<T>)."))
                    }
                } else {
                    Err(syn::Error::new_spanned(ty, "Vec must have angle-bracketed generic arguments."))
                }
            } else if ident_str == "HashMap" || ident_str == "BTreeMap" {
                // Validate key type is String and recursively validate value type
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if args.args.len() == 2 {
                        if let (Some(GenericArgument::Type(key_ty)), Some(GenericArgument::Type(value_ty))) =
                            (args.args.first(), args.args.get(1))
                        {
                            if let Type::Path(key_path) = key_ty {
                                if let Some(key_segment) = key_path.path.segments.last() {
                                    if key_segment.ident != "String" {
                                        return Err(syn::Error::new_spanned(key_ty, "HashMap/BTreeMap key must be `String` for GitOps resources."));
                                    }
                                } else {
                                     return Err(syn::Error::new_spanned(key_ty, "HashMap/BTreeMap key must be `String` for GitOps resources."));
                                }
                            } else {
                                return Err(syn::Error::new_spanned(key_ty, "HashMap/BTreeMap key must be `String` for GitOps resources."));
                            }
                            validate_gitops_field_type(value_ty) // Recursive call for value type
                        } else {
                            Err(syn::Error::new_spanned(ty, "HashMap/BTreeMap must have two generic arguments (e.g., HashMap<K, V>)."))
                        }
                    } else {
                        Err(syn::Error::new_spanned(ty, "HashMap/BTreeMap must have two generic arguments (e.g., HashMap<K, V>)."))
                    }
                } else {
                    Err(syn::Error::new_spanned(ty, "HashMap/BTreeMap must have angle-bracketed generic arguments."))
                }
            }
            // For any other `Type::Path`, we assume it's a struct/enum meant to be a `GitopsResourcePart`.
            // The compiler will later ensure it actually implements `GitopsResourcePart`.
            else {
                Ok(())
            }
        }
        _ => Err(syn::Error::new_spanned(ty, "Unsupported type for GitOps resource field. Only primitive types, String, Option<T>, Vec<T>, HashMap<String, V>, and other GitopsResourcePart-annotated structs/enums are allowed.")),
    }
}

/// Helper to check if a type is a "part-like" type, meaning it's a struct/enum
/// that *should* be annotated with `GitopsResourcePart`. This is a heuristic
/// for generating merge logic, as proc macros cannot check trait implementations.
fn is_gitops_part_like_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident_str = segment.ident.to_string();
            // It's "part-like" if it's not a primitive, String, Option, Vec, or HashMap/BTreeMap
            ![
                "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64",
                "bool", "char", "String", "Option", "Vec", "HashMap", "BTreeMap",
            ]
            .contains(&ident_str.as_str())
        } else {
            false
        }
    } else {
        // If it's not a path type (e.g., a reference, array), it's not a "part-like" struct.
        false
    }
}

/// Helper function to extract the last Ident from a TypePath.
/// This is typically used to get the struct/enum name from its type.
fn get_ident_from_type_path(ty: &Type) -> Option<&Ident> {
    if let Type::Path(type_path) = ty {
        type_path.path.segments.last().map(|s| &s.ident)
    } else {
        None
    }
}

/// Helper to get the inner type of an `Option<T>`.
fn get_option_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

/// Helper to get the inner type of a `Vec<T>`.
fn get_vec_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Vec" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

/// Implements `GitopsResourceRoot` for a struct.
pub fn gitops_resource_root_derive_impl(
    input: syn::DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Struct(data_struct) = &input.data else {
        return Err(syn::Error::new_spanned(
            input.ident,
            "GitopsResourceRoot can only be derived for structs.",
        ));
    };

    let fields = &data_struct.fields;
    let named_fields = fields.iter().collect::<Vec<_>>();

    // Parse macro attributes for GitopsResourceRoot
    let mut key_field_ident: Option<Ident> = None;
    let mut api_version = "v1.0".to_string(); // Default apiVersion

    for attr in &input.attrs {
        if attr.path().is_ident("gitops") {
            let parsed_meta_list = attr.parse_args_with(GitopsAttributeArgs::parse)?;
            for nested_meta in parsed_meta_list.args {
                if let Meta::NameValue(MetaNameValue { path, value, .. }) = nested_meta {
                    let lit_str: LitStr = syn::parse2(value.to_token_stream())?;
                    if path.is_ident("key") {
                        key_field_ident = Some(format_ident!("{}", lit_str.value()));
                    } else if path.is_ident("api_version") {
                        api_version = lit_str.value();
                    }
                } else {
                    return Err(syn::Error::new_spanned(nested_meta, "Unexpected nested attribute format. Expected `key = \"...\"` or `api_version = \"...\"`."));
                }
            }
        }
    }

    let key_field_ident = key_field_ident.ok_or_else(|| {
        syn::Error::new_spanned(
            struct_name,
            "GitopsResourceRoot requires a `key` attribute, e.g., #[gitops(key = \"id\")]",
        )
    })?;

    // Validate key field exists and is of String type
    let key_field: &Field = named_fields
        .iter()
        .find(|f| f.ident.as_ref() == Some(&key_field_ident))
        .ok_or_else(|| {
            syn::Error::new_spanned(
                &key_field_ident,
                format!(
                    "Key field `{}` not found in struct `{}`.",
                    key_field_ident, struct_name
                ),
            )
        })?;

    if let Type::Path(ty_path) = &key_field.ty {
        if let Some(segment) = ty_path.path.segments.last() {
            if segment.ident != "String" {
                return Err(syn::Error::new_spanned(
                    &key_field.ty,
                    "The key field specified by `key` attribute must be of type `String`.",
                ));
            }
        } else {
            return Err(syn::Error::new_spanned(
                &key_field.ty,
                "The key field specified by `key` attribute must be of type `String`.",
            ));
        }
    } else {
        return Err(syn::Error::new_spanned(
            &key_field.ty,
            "The key field specified by `key` attribute must be of type `String`.",
        ));
    }

    // Names for generated structs
    let serializable_struct_name = format_ident!("{}GitopsSerializable", struct_name);
    let update_struct_name = format_ident!("{}GitopsUpdate", struct_name);
    let kind_value = struct_name.to_string(); // 'kind' is the struct name itself

    // Collect fields for the generated serializable struct (simple copy)
    let serializable_fields: Vec<TokenStream> = named_fields
        .iter()
        .map(|f| {
            let field_name_ident = f.ident.as_ref().expect("Expected named field").clone();
            let field_type = &f.ty;
            let field_vis = &f.vis;
            quote! {
                #field_vis #field_name_ident: #field_type,
            }
        })
        .collect();

    // Field initializers for `From<Resource> for Serializable`
    let from_resource_initializers: Vec<TokenStream> = named_fields
        .iter()
        .map(|f| {
            let field_name_ident = f.ident.as_ref().expect("Expected named field").clone();
            quote! { #field_name_ident: resource.#field_name_ident, }
        })
        .collect();

    // Field initializers for `From<Serializable> for Resource`
    let from_serializable_initializers: Vec<TokenStream> = named_fields
        .iter()
        .map(|f| {
            let field_name_ident = f.ident.as_ref().expect("Expected named field").clone();
            quote! { #field_name_ident: serializable_resource.#field_name_ident, }
        })
        .collect();

    // Field initializers for `as_serializable` method
    let as_serializable_fields: Vec<TokenStream> = named_fields
        .iter()
        .map(|f| {
            let field_name_ident = f.ident.as_ref().expect("Expected named field").clone();
            quote! { #field_name_ident: self.#field_name_ident.clone(), }
        })
        .collect();

    // Field initializers for `into_serializable` method
    let into_serializable_fields: Vec<TokenStream> = named_fields
        .iter()
        .map(|f| {
            let field_name_ident = f.ident.as_ref().expect("Expected named field").clone();
            quote! { #field_name_ident: self.#field_name_ident, }
        })
        .collect();

    // Collect fields for the generated update struct and merging logic
    let mut update_struct_fields = Vec::new();
    let mut merge_logic_updates = Vec::new(); // Logic for `with_updates_from`

    for field in named_fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let field_vis = &field.vis;

        // Validate field type against GitOps rules
        validate_gitops_field_type(field_type)?;

        // Check for special field attributes
        let mut skip_on_update = false;
        let mut required_in_update = false;

        for attr in &field.attrs {
            if attr.path().is_ident("gitops") {
                let parsed_meta_list = attr.parse_args_with(GitopsAttributeArgs::parse)?;
                for nested_meta in parsed_meta_list.args {
                    if let Meta::Path(path) = nested_meta {
                        if path.is_ident("skip_on_update") {
                            skip_on_update = true;
                        } else if path.is_ident("required_in_update") {
                            required_in_update = true;
                        }
                    } else {
                        return Err(syn::Error::new_spanned(nested_meta, "Unexpected nested attribute format. Expected `skip_on_update` or `required_in_update`."));
                    }
                }
            }
        }

        let mut update_field_type_tokens = quote! { #field_type }; // Default to original type

        // Determine the type to use in the `_Update` struct
        if required_in_update {
            // If required in update, the field type is the original type.
            // If it's a part, we still use the part's original type here, and its `with_updates_from_part` is called.
            if is_gitops_part_like_type(field_type) {
                let field_ty_ident = get_ident_from_type_path(field_type).ok_or_else(|| {
                    syn::Error::new_spanned(
                        field_type,
                        "Expected Ident for GitopsResourcePart type.",
                    )
                })?;
                let update_name = format_ident!("{}GitopsUpdate", field_ty_ident);
                update_field_type_tokens = quote! { #update_name };
            } else {
                update_field_type_tokens = quote! { #field_type };
            }
        } else {
            // Default behavior: field is optional in update, wrapped in Option.
            if let Some(inner_ty) = get_option_inner_type(field_type) {
                // Original field is Option<T>
                if is_gitops_part_like_type(inner_ty) {
                    // Original: Option<Part>, Update: Option<Option<Part>>
                    let inner_ty_ident = get_ident_from_type_path(inner_ty).ok_or_else(|| {
                        syn::Error::new_spanned(
                            inner_ty,
                            "Expected Ident for GitopsResourcePart inner type.",
                        )
                    })?;
                    // let inner_update_name = format_ident!("{}GitopsUpdate", inner_ty_ident);
                    update_field_type_tokens = quote! { Option<#inner_ty> };
                } else {
                    // Original: Option<Primitive/Vec/HashMap>, Update: Option<Option<Primitive/Vec/HashMap>>
                    update_field_type_tokens = quote! { #field_type };
                }
            } else {
                // Original field is T (not an Option)
                if is_gitops_part_like_type(field_type) {
                    // Original: Part, Update: Option<PartUpdate>
                    let field_ty_ident = get_ident_from_type_path(field_type).ok_or_else(|| {
                        syn::Error::new_spanned(
                            field_type,
                            "Expected Ident for GitopsResourcePart type.",
                        )
                    })?;
                    let update_name = format_ident!("{}GitopsUpdate", field_ty_ident);
                    update_field_type_tokens = quote! { #update_name };
                } else {
                    // Original: Primitive/Vec/HashMap, Update: Option<Primitive/Vec/HashMap>
                    update_field_type_tokens = quote! { #field_type };
                }
            }
            // Always wrap in an Option for non-required fields in the Update struct
            update_field_type_tokens = quote! { Option<#update_field_type_tokens> };
        }

        if field_name == &key_field_ident {
            // Key field is included in update struct as original type (for matching)
            update_struct_fields.push(quote! {
                #field_vis #field_name: #field_type,
            });
            // Key field is NEVER updated
            merge_logic_updates.push(quote! {
                // Key field is explicitly skipped from updates. Its presence in `updates` is for identification.
            });
        } else if skip_on_update {
            // Field marked to be skipped from updates
            update_struct_fields.push(quote! {
                #[serde(skip_serializing, skip_deserializing)] // Not meant for external updates
                #field_vis #field_name: #field_type, // Stored but not serialized/deserialized for update
            });
            merge_logic_updates.push(quote! {
                // Field with `skip_on_update` is explicitly skipped from updates.
            });
        } else if required_in_update {
            // Field required in update.
            update_struct_fields.push(quote! {
                #field_vis #field_name: #update_field_type_tokens,
            });
            // For required_in_update fields, they are replaced.
            // If it's a GitopsResourcePart, we call its `with_updates_from_part` for deep merge.
            // If it's not a part, it's a primitive/collection, so it's replaced.
            if is_gitops_part_like_type(field_type) {
                merge_logic_updates.push(quote! {
                    updated.#field_name = self.#field_name.with_updates_from_part(updates.#field_name);
                });
            } else {
                merge_logic_updates.push(quote! {
                    updated.#field_name = updates.#field_name;
                });
            }
        } else {
            // Default: wrap in Option and apply if Some.
            update_struct_fields.push(quote! {
                #[serde(default, skip_serializing_if = "Option::is_none")]
                #field_vis #field_name: #update_field_type_tokens,
            });

            // Merge logic: check if `Some` and update.
            // Handles Option<GitopsResourcePart> for deep merging.
            if let Some(original_inner_ty) = get_option_inner_type(field_type) {
                if is_gitops_part_like_type(original_inner_ty) {
                    // If original is Option<GitopsResourcePart> -> update is Option<Option<GitopsResourcePartUpdate>>
                    merge_logic_updates.push(quote! {
                        if let Some(new_outer_val) = updates.#field_name { // new_outer_val is Option<InnerTypeGitopsUpdate>
                            if let Some(new_inner_val) = new_outer_val { // new_inner_val is InnerTypeGitopsUpdate
                                if let Some(current_val) = updated.#field_name.take() {
                                    // Deep merge if current value exists
                                    updated.#field_name = Some(gitops_lib::GitopsResourcePart::with_updates_from_part(current_val, gitops_lib::GitopsResourcePart::as_update(&new_inner_val)));
                                } else {
                                    // Replace if no current value
                                    updated.#field_name = Some(new_inner_val.into());
                                }
                            } else {
                                // If update provides `Some(None)`, set to `None`
                                updated.#field_name = None;
                            }
                        }
                    });
                } else {
                    // If original is Option<Primitive/Vec/HashMap> -> update is Option<Option<Primitive/Vec/HashMap>>
                    merge_logic_updates.push(quote! {
                        if let Some(new_value) = updates.#field_name {
                            // Simply replace the Option itself. new_value is Option<T>
                            updated.#field_name = new_value;
                        }
                    });
                }
            } else {
                // Original field type is T (not an Option) -> update field is Option<TGitopsUpdate> or Option<T>
                if is_gitops_part_like_type(field_type) {
                    // If original is GitopsResourcePart (not Option) -> update is Option<GitopsResourcePartUpdate>
                    merge_logic_updates.push(quote! {
                        if let Some(new_value) = updates.#field_name { // new_value is PartGitopsUpdate
                            updated.#field_name = updated.#field_name.with_updates_from_part(new_value);
                        }
                    });
                } else {
                    // If original is Primitive/Vec/HashMap (not Option) -> update is Option<Primitive/Vec/HashMap>
                    merge_logic_updates.push(quote! {
                        if let Some(new_value) = updates.#field_name {
                            updated.#field_name = new_value; // Direct assignment for non-Option fields
                        }
                    });
                }
            }
        }
    }

    let r#gen = quote! {
        // Automatically derive Clone for the original struct
        // IMPORTANT: The original struct definition is NOT re-emitted here.
        // It is provided by the user's code. Only the impl blocks are generated.

        impl #impl_generics From<#struct_name #ty_generics> for #serializable_struct_name #ty_generics #where_clause {
            fn from(resource: #struct_name #ty_generics) -> Self {
                let my_datetime: ::chrono::DateTime<::chrono::Utc> = ::chrono::Utc::now();
                let timestamp_secs: i64 = my_datetime.timestamp();
                // generate default timestamp: now
                Self {
                    kind: #kind_value.to_string(),
                    api_version: #api_version.to_string(),
                    mod_timestamp: timestamp_secs,
                    #(#from_resource_initializers)*
                }
            }
        }

        impl #impl_generics From<#serializable_struct_name #ty_generics> for #struct_name #ty_generics #where_clause {
            fn from(serializable_resource: #serializable_struct_name #ty_generics) -> Self {
                Self {
                    #(#from_serializable_initializers)*
                }
            }
        }


        // Generated Update Struct
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct #update_struct_name #ty_generics #where_clause {
            #(#update_struct_fields)*
        }

        // Implementation of GitopsResourceRoot trait
        impl #impl_generics gitops_lib::GitopsResourceRoot for #struct_name #ty_generics #where_clause {
            type Serializable = #serializable_struct_name #ty_generics;
            type Update = #update_struct_name #ty_generics;

            fn as_serializable(&self) -> Self::Serializable {
                let now = ::chrono::Utc::now();
                Self::Serializable {
                    kind: #kind_value.to_string(),
                    api_version: #api_version.to_string(),
                    mod_timestamp: now.timestamp(),
                    #(#as_serializable_fields)*
                }
            }

            fn into_serializable(self) -> Self::Serializable {
                let now = ::chrono::Utc::now();
                Self::Serializable {
                    kind: #kind_value.to_string(),
                    api_version: #api_version.to_string(),
                    mod_timestamp: now.timestamp(),
                    #(#into_serializable_fields)*
                }
            }

            fn into_serializable_with_timestamp(self, timestamp: i64) -> Self::Serializable {
                Self::Serializable {
                    kind: #kind_value.to_string(),
                    api_version: #api_version.to_string(),
                    mod_timestamp: timestamp,
                    #(#into_serializable_fields)*
                }
            }

            fn as_serializable_with_timestamp(&self, timestamp: i64) -> Self::Serializable {
                Self::Serializable {
                    kind: #kind_value.to_string(),
                    api_version: #api_version.to_string(),
                    mod_timestamp: timestamp,
                    #(#as_serializable_fields)*
                }
            }

            fn get_kind(&self) -> String {
                #kind_value.to_string()
            }

            fn get_key(&self) -> String {
                self.#key_field_ident.clone()
            }

            fn kind() -> &'static str {
                #kind_value
            }

            fn with_updates_from(self, updates: Self::Update) -> Self {
                // Ensure the key matches before attempting to merge
                if self.#key_field_ident != updates.#key_field_ident {
                    panic!("Attempted to merge updates from an object with a different key. Current key: {}, Update key: {}", self.#key_field_ident, updates.#key_field_ident);
                }

                let mut updated = self; // Start with the original struct (consumed by `self`)

                // Update the fields based on `updates`
                #(#merge_logic_updates)*

                updated
            }
        }

        // Generated Serializable Struct (defined here because it's used in impl block above)
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")] // Common GitOps API convention
        #[allow(non_snake_case)] // Allow non-snake_case for kind and apiVersion if serde renames
        pub struct #serializable_struct_name #ty_generics #where_clause {
            pub kind: String,
            #[serde(rename = "apiVersion")]
            pub api_version: String,
            pub mod_timestamp: i64,
            #(#serializable_fields)*
        }
    };
    Ok(r#gen.into())
}

/// Implements `GitopsResourcePart` for a struct.
pub fn gitops_resource_part_derive_impl(
    input: syn::DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Struct(data_struct) = &input.data else {
        return Err(syn::Error::new_spanned(
            input.ident,
            "GitopsResourcePart can only be derived for structs.",
        ));
    };

    let fields = &data_struct.fields;
    let named_fields = fields.iter().collect::<Vec<_>>();

    // Validate all fields
    for field in named_fields.iter() {
        validate_gitops_field_type(&field.ty)?;
    }

    // Generate merging logic for GitopsResourcePart's `with_updates_from_part`
    let mut part_merge_logic = Vec::new();
    for field in named_fields.iter() {
        let field_name_ident = field.ident.as_ref().expect("Expected named field").clone();
        let field_type = &field.ty;
        let field_vis = &field.vis;

        // Check for special field attributes for update struct generation
        let mut required_in_update = false;
        for attr in &field.attrs {
            if attr.path().is_ident("gitops") {
                let parsed_meta_list = attr.parse_args_with(GitopsAttributeArgs::parse)?;
                for nested_meta in parsed_meta_list.args {
                    if let Meta::Path(path) = nested_meta {
                        if path.is_ident("required_in_update") {
                            required_in_update = true;
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            nested_meta,
                            "Unexpected nested attribute format. Expected `required_in_update`.",
                        ));
                    }
                }
            }
        }

        if required_in_update {
            // For required fields, always replace
            if is_gitops_part_like_type(field_type) {
                // Original field is a Part, and update field is the Part itself (not Option)
                part_merge_logic.push(quote! {
                    updated.#field_name_ident = updated.#field_name_ident.with_updates_from_part(updates.#field_name_ident);
                });
            } else {
                // Original field is primitive/collection, and update field is the type itself (not Option)
                part_merge_logic.push(quote! {
                    updated.#field_name_ident = updates.#field_name_ident;
                });
            }
        } else {
            // Default case: field in update struct is Option<OriginalType>
            // Here, updates.#field_name_ident is `Option<OriginalType>` for non-Option fields,
            // and `Option<Option<OriginalInnerType>>` for Option fields.

            if let Some(original_inner_ty) = get_option_inner_type(field_type) {
                // Original field is Option<T> -> update field is Option<Option<T>>
                // We need to unpack two layers of Option.
                if is_gitops_part_like_type(original_inner_ty) {
                    // Case: Option<GitopsResourcePart> -> Option<Option<GitopsResourcePartUpdate>>
                    part_merge_logic.push(quote! {
                        if let Some(new_outer_val) = updates.#field_name_ident { // new_outer_val is Option<InnerTypeGitopsUpdate>
                            if let Some(new_inner_val) = new_outer_val { // new_inner_val is InnerTypeGitopsUpdate
                                if let Some(current_val) = updated.#field_name_ident.take() {
                                    // Deep merge if current value exists
                                    updated.#field_name_ident = Some(current_val.with_updates_from_part(new_inner_val));
                                } else {
                                    // Replace if no current value
                                    updated.#field_name_ident = Some(new_inner_val.into()); // Assuming From<Update> for Original exists
                                }
                            } else {
                                // If update provides `Some(None)`, set to `None`
                                updated.#field_name_ident = None;
                            }
                        }
                    });
                } else {
                    // If original is Option<Primitive/Vec/HashMap> -> update is Option<Option<Primitive/Vec/HashMap>>
                    part_merge_logic.push(quote! {
                        if let Some(new_value) = updates.#field_name_ident {
                            // Simply replace the Option itself. new_value is Option<T>
                            updated.#field_name_ident = new_value;
                        }
                    });
                }
            } else {
                // Original field is T (not an Option) -> update field is Option<TGitopsUpdate> or Option<T>
                if is_gitops_part_like_type(field_type) {
                    // If original is GitopsResourcePart (not Option) -> update is Option<GitopsResourcePartUpdate>
                    part_merge_logic.push(quote! {
                        if let Some(new_value) = updates.#field_name_ident {
                            updated.#field_name_ident = updated.#field_name_ident.with_updates_from_part(new_value);
                        }
                    });
                } else {
                    // If original is Primitive/Vec/HashMap (not Option) -> update is Option<Primitive/Vec/HashMap>
                    part_merge_logic.push(quote! {
                        if let Some(new_value) = updates.#field_name_ident {
                            updated.#field_name_ident = new_value; // Direct assignment for non-Option fields
                        }
                    });
                }
            }
        }
    }

    // Generated Update Struct for Part to enable recursive merging for nested parts
    let part_update_struct_name = format_ident!("{}GitopsUpdate", struct_name);
    let mut part_update_struct_fields = Vec::new();

    for field in named_fields.iter() {
        let field_name_ident = field.ident.as_ref().expect("Expected named field").clone();
        let field_type = &field.ty;
        let field_vis = &field.vis;

        let mut required_in_update = false;
        for attr in &field.attrs {
            if attr.path().is_ident("gitops") {
                let parsed_meta_list = attr.parse_args_with(GitopsAttributeArgs::parse)?;
                for nested_meta in parsed_meta_list.args {
                    if let Meta::Path(path) = nested_meta {
                        if path.is_ident("required_in_update") {
                            required_in_update = true;
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            nested_meta,
                            "Unexpected nested attribute format. Expected `required_in_update`.",
                        ));
                    }
                }
            }
        }

        let mut update_field_type_tokens = quote! { #field_type }; // Default to original type

        // Determine the type to use in the `_Update` struct
        if required_in_update {
            // If required in update, the field type is the original type.
            // If it's a part, it becomes the Part's generated update type.
            if is_gitops_part_like_type(field_type) {
                let field_ty_ident = get_ident_from_type_path(field_type).ok_or_else(|| {
                    syn::Error::new_spanned(
                        field_type,
                        "Expected Ident for GitopsResourcePart type.",
                    )
                })?;
                let update_name = format_ident!("{}GitopsUpdate", field_ty_ident);
                update_field_type_tokens = quote! { #update_name };
            } else {
                update_field_type_tokens = quote! { #field_type };
            }
        } else {
            // Default behavior: field is optional in update, wrapped in Option.
            if let Some(inner_ty) = get_option_inner_type(field_type) {
                // Original field is Option<T>
                if is_gitops_part_like_type(inner_ty) {
                    // Original: Option<Part>, Update: Option<Option<PartUpdate>>
                    let inner_ty_ident = get_ident_from_type_path(inner_ty).ok_or_else(|| {
                        syn::Error::new_spanned(
                            inner_ty,
                            "Expected Ident for GitopsResourcePart inner type.",
                        )
                    })?;
                    let inner_update_name = format_ident!("{}GitopsUpdate", inner_ty_ident);
                    update_field_type_tokens = quote! { Option<#inner_update_name> };
                } else {
                    // Original: Option<Primitive/Vec/HashMap>, Update: Option<Option<Primitive/Vec/HashMap>>
                    update_field_type_tokens = quote! { #field_type };
                }
            } else {
                // Original field is T (not an Option)
                if is_gitops_part_like_type(field_type) {
                    // Original: Part, Update: Option<PartUpdate>
                    let field_ty_ident = get_ident_from_type_path(field_type).ok_or_else(|| {
                        syn::Error::new_spanned(
                            field_type,
                            "Expected Ident for GitopsResourcePart type.",
                        )
                    })?;
                    let update_name = format_ident!("{}GitopsUpdate", field_ty_ident);
                    update_field_type_tokens = quote! { #update_name };
                } else {
                    // Original: Primitive/Vec/HashMap, Update: Option<Primitive/Vec/HashMap>
                    update_field_type_tokens = quote! { #field_type };
                }
            }
            // Always wrap in an Option for non-required fields in the Update struct
            update_field_type_tokens = quote! { Option<#update_field_type_tokens> };
        }

        part_update_struct_fields.push(quote! {
        #field_vis #field_name_ident: #update_field_type_tokens,
        });
    }

    // Generate `as_update` method
    let as_update_initializers: Vec<TokenStream> = named_fields
        .iter()
        .map(|f| {
            let field_name_ident = f.ident.as_ref().expect("Expected named field").clone();
            let field_type = &f.ty;

            let mut skip_on_update = false;
            let mut required_in_update = false;
            for attr in &f.attrs {
                if attr.path().is_ident("gitops") {
                    if let Ok(parsed_meta_list) = attr.parse_args_with(GitopsAttributeArgs::parse) {
                        for nested_meta in parsed_meta_list.args {
                            if let Meta::Path(path) = nested_meta {
                                if path.is_ident("skip_on_update") {
                                    skip_on_update = true;
                                } else if path.is_ident("required_in_update") {
                                    required_in_update = true;
                                }
                            }
                        }
                    }
                }
            }

            let init_expr = if skip_on_update {
                // Field is present in update struct but skipped for serialization/deserialization.
                // Just clone the original value.
                quote! { self.#field_name_ident.clone() }
            } else if required_in_update {
                // Field is T in original, T_Update in update.
                if is_gitops_part_like_type(field_type) {
                    // Original: Part, Update: PartUpdate
                    quote! { self.#field_name_ident.as_update() }
                } else {
                    // Original: Primitive/Vec/HashMap, Update: Primitive/Vec/HashMap
                    quote! { self.#field_name_ident.clone() }
                }
            } else {
                // Field is T or Option<T> in original, Option<T_Update> or Option<Option<T_Update>> in update.
                if let Some(original_inner_ty) = get_option_inner_type(field_type) {
                    // Original field is Option<T>
                    if is_gitops_part_like_type(original_inner_ty) {
                        // Original: Option<Part>, Update: Option<Option<PartUpdate>>
                        quote! { self.#field_name_ident.as_ref().map(|x| x.as_update()) }
                    } else {
                        // Original: Option<Primitive/Vec/HashMap>, Update: Option<Option<Primitive/Vec/HashMap>>
                        quote! { self.#field_name_ident.clone() }
                    }
                } else {
                    // Original field is T (not Option)
                    if is_gitops_part_like_type(field_type) {
                        // Original: Part, Update: Option<PartUpdate>
                        quote! { Some(self.#field_name_ident.as_update()) }
                    } else {
                        // Original: Primitive/Vec/HashMap, Update: Option<Primitive/Vec/HashMap>
                        quote! { Some(self.#field_name_ident.clone()) }
                    }
                }
            };
            quote! { #field_name_ident: #init_expr, }
        })
        .collect();

    let r#gen = quote! {
        // IMPORTANT: The original struct definition is NOT re-emitted here.
        // It is provided by the user's code. Only the impl blocks and generated structs are emitted.
        // The user must manually apply `#[derive(Clone, Debug, Serialize, Deserialize)]` and
        // `#[serde(rename_all = "camelCase")]` to their original struct.

        // Generated Update Struct for Part (for recursive merging)
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct #part_update_struct_name #ty_generics #where_clause {
            #(#part_update_struct_fields)*
        }


        // Implement GitopsResourcePart trait
        impl #impl_generics gitops_lib::GitopsResourcePart for #struct_name #ty_generics #where_clause {
            // The 'Update' associated type is removed from the trait.
            // The generated `with_updates_from_part` will directly use the concrete struct name.
            fn with_updates_from_part(self, updates: Self::UpdatePart) -> Self { // Directly use the generated struct name
                let mut updated = self;

                #(#part_merge_logic)* // Apply updates

                updated
            }

            fn as_update(&self) -> Self::UpdatePart {
                #part_update_struct_name {
                    #(#as_update_initializers)*
                }
            }

            type UpdatePart = #part_update_struct_name #ty_generics;
        }

        // Add a helper method for deep merging nested GitopsResourcePart fields.
        // This is called by the root's `with_updates_from` for nested parts.
        impl #impl_generics #struct_name #ty_generics #where_clause {

        }
    };
    Ok(r#gen.into())
}

pub fn gitops_enum_derive_impl(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let enum_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new_spanned(
            input.ident,
            "GitopsEnum can only be derived for enums.",
        ));
    };

    // Validate that all variants are unit variants (no associated data)
    for variant in &data_enum.variants {
        if !variant.fields.is_empty() {
            return Err(syn::Error::new_spanned(
                &variant.fields,
                "GitopsEnum only supports simple enums without associated values (unit variants).",
            ));
        }
    }

    let r#gen = quote! {
        // Implement GitopsResourcePart trait for the enum
        impl #impl_generics gitops_lib::GitopsResourcePart for #enum_name #ty_generics #where_clause {
            type UpdatePart = Self; // For enums, the update type is the enum itself

            // For simple enums, update is a direct replacement, so `updates` is `Self`.
            fn with_updates_from_part(self, updates: Self::UpdatePart) -> Self {
                updates // A simple enum is replaced entirely by its update
            }

            // For simple enums, the enum itself acts as its own update representation.
            fn as_update(&self) -> Self::UpdatePart {
                self.clone()
            }
        }
    };

    Ok(r#gen.into())
}
