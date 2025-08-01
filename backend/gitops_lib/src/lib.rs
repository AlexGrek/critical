pub use gitops_macros::{GitopsResourcePart, GitopsResourceRoot, GitopsEnum};
use serde::{Deserialize, Serialize};

pub mod store;

pub trait GitopsSerializable: Sized + Clone + Send + Sync + 'static {

}

/// Trait for root-level GitOps resources.
/// Implementations are generated by the `#[derive(GitopsResourceRoot)]` macro.
pub trait GitopsResourceRoot: Sized + Clone + std::fmt::Debug + Send + Sync + 'static + From<Self::Serializable> {
    /// Converts the resource into its serializable representation.
    fn as_serializable(&self) -> Self::Serializable;
    /// Consumes the resource and converts it into its serializable representation.
    fn into_serializable(self) -> Self::Serializable;

    fn as_serializable_with_timestamp(&self, timestamp: i64) -> Self::Serializable;

    fn into_serializable_with_timestamp(self, timestamp: i64) -> Self::Serializable;
    
    /// Applies updates from an update struct, returning a new resource instance.
    /// This method consumes `self`.
    fn with_updates_from(self, updates: Self::Update) -> Self;

    fn get_key(&self) -> String;

    fn get_kind(&self) -> String;

    fn kind() -> &'static str;

    /// Associated type for the generated serializable struct.
    type Serializable: std::fmt::Debug + Serialize + for<'de> Deserialize<'de>;
    /// Associated type for the generated update struct.
    type Update: std::fmt::Debug + Serialize + for<'de> Deserialize<'de>;
}

/// Trait for parts of GitOps resources (nested structs).
/// Implementations are generated by the `#[derive(GitopsResourcePart)]` macro.
pub trait GitopsResourcePart: Sized + Clone + std::fmt::Debug + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {
    /// Associated type for the update representation of this part.
    /// For enums, this will typically be `Self`. For structs, it will be a generated `MyStructGitopsUpdate` struct.
    type UpdatePart: Sized + std::fmt::Debug + serde::Serialize + for<'de> serde::Deserialize<'de>;

    /// Merges updates from another part instance into this one.
    /// This method consumes `self`.
    fn with_updates_from_part(self, updates: Self::UpdatePart) -> Self;

    /// Creates an update representation of this part.
    /// For simple enums, this is `Self`. For structs, it's the generated `_Update` struct.
    fn as_update(&self) -> Self::UpdatePart;
}

// Private module for helpers that might be used by the generated macro code
#[doc(hidden)]
pub mod private {
    use syn::Type;

    /// Helper used by the macro to check if a type is "part-like".
    /// This is a mirror of the `is_gitops_part_like_type` in `gitops_macros`.
    /// It must be kept in sync.
    pub fn is_gitops_part_like_type_check(ty: &syn::Type) -> bool {
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                let ident_str = segment.ident.to_string();
                !["u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64", "bool", "char", "String", "Option", "Vec", "HashMap", "BTreeMap"]
                    .contains(&ident_str.as_str())
            } else {
                false
            }
        } else {
            false
        }
    }
}
