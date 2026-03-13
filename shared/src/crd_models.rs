use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Custom Resource Definitions (CRDs)
// ---------------------------------------------------------------------------

/// Scope determines where instances of this resource live.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CrdScope {
    /// Cluster-wide resource, accessed at `/v1/global/{kind}`.
    #[default]
    Global,
    /// Namespaced under a project, accessed at `/v1/projects/{project}/{kind}`.
    Project,
}

/// Controls how ACLs are applied to instances of this resource kind.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CrdAclMode {
    /// No ACL field on instances; access controlled exclusively by super-permissions.
    /// Equivalent to `#[crit_resource(no_acl)]`.
    #[default]
    Special,
    /// Instances do not carry their own ACL; permission checks fall back to
    /// the parent project's ACL (project-scoped only).
    Inherit,
    /// Each instance carries its own `AccessControlStore`.
    Custom,
}

/// Human-readable noun forms used in CLI output, API error messages, and UI.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CrdNouns {
    /// Singular form, e.g. `"deployment"`.
    pub singular: String,
    /// Plural form, e.g. `"deployments"`. Also used as the API path segment.
    pub plural: String,
}

/// An edge relation that instances of this CRD participate in.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrdRelation {
    /// ArangoDB edge collection name, e.g. `"deploys_to"`.
    pub edge_collection: String,
    /// Direction from this resource's perspective.
    pub direction: RelationDirection,
    /// The kind (CRD name or built-in kind) on the other end.
    pub target_kind: String,
    /// Human label for the relation, e.g. `"targets environment"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationDirection {
    /// This resource is the `_from` vertex.
    Outbound,
    /// This resource is the `_to` vertex.
    Inbound,
}

// ---------------------------------------------------------------------------
// Field definitions (recursive)
// ---------------------------------------------------------------------------

/// Primitive and composite types available for CRD fields.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum FieldType {
    /// UTF-8 string.
    String,
    /// Signed 64-bit integer.
    Int,
    /// 64-bit IEEE 754 float.
    Float,
    /// Boolean.
    Bool,
    /// RFC 3339 date-time (stored as ISO 8601 string, parsed as `DateTime<Utc>`).
    Datetime,
    /// Resource identifier — validated with the same rules as built-in IDs
    /// (alphanumeric + `_` + `-`, no leading digit, max 63 chars).
    Id,
    /// Ordered list of a single element type.
    List {
        /// Type of each list element.
        items: Box<FieldType>,
    },
    /// String-keyed map where every value shares the same type.
    Map {
        /// Type of each map value.
        values: Box<FieldType>,
    },
    /// Enumeration — value must be one of the listed strings.
    Enum {
        /// Allowed values (non-empty).
        variants: Vec<String>,
    },
    /// Bitflags stored as a `u64`. Each named flag maps to a single bit.
    /// Serialized as an integer; the `bit_positions` map provides human-readable names.
    /// The map value is the *bit position* (0-63), not the bit *value* (1 << position).
    /// For example, `{"can_deploy": 0, "can_rollback": 1}` means the first two bits.
    Bitflags {
        /// Map of flag name → bit position (0-63). Bit value = 1 << bit_position.
        bit_positions: HashMap<String, u8>,
    },
    /// Inline object with its own named fields (recursive).
    Object {
        /// Nested field definitions.
        fields: HashMap<String, FieldDef>,
    },
    /// Reference to another resource by kind + ID.
    /// Stored as a plain string (the referenced resource's ID).
    Ref {
        /// Target kind name, e.g. `"users"` or another CRD name.
        kind: String,
    },
}

/// A single field inside a CRD schema.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldDef {
    /// The type (and any nested schema) for this field.
    #[serde(flatten)]
    pub field_type: FieldType,
    /// When `true`, the field must be present and non-null on create/update.
    #[serde(default)]
    pub required: bool,
    /// Default value (JSON). Applied server-side when the field is absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Human-readable description (for docs / UI tooltips).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Include this field in the `{Name}Brief` list projection.
    #[serde(default)]
    pub brief: bool,
    /// Make this field immutable after creation (server rejects changes).
    #[serde(default)]
    pub immutable: bool,
}

// ---------------------------------------------------------------------------
// The CRD itself
// ---------------------------------------------------------------------------

/// A Custom Resource Definition describes a new resource kind that the system
/// manages dynamically. CRDs are themselves stored as documents in the `crds`
/// collection, and their instances live in the collection named by `id`.
///
/// CRDs are fully describable as YAML or JSON — no Rust code generation
/// required. The generic gitops handlers use the CRD schema at runtime for
/// validation, serialization, and ACL enforcement.
#[crit_derive::crit_resource(collection = "crds", prefix = "", no_acl)]
pub struct CustomResourceDefinition {
    /// Where instances of this resource live.
    pub scope: CrdScope,

    /// How ACLs are applied to instances.
    pub acl_mode: CrdAclMode,

    /// Human-readable noun forms for CLI/UI.
    pub nouns: CrdNouns,

    /// Edge relations this resource participates in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<CrdRelation>,

    /// Schema: named fields that instances of this kind carry.
    /// Standard `#[crit_resource]` fields (id, labels, annotations, state,
    /// acl, deletion, hash_code) are injected automatically — do NOT
    /// re-declare them here.
    pub fields: HashMap<String, FieldDef>,

    /// Optional ID prefix for instances (e.g. `"dep_"` for deployments).
    /// Empty string means no prefix (like projects).
    #[serde(default)]
    pub id_prefix: String,

    /// Optional super-permission that short-circuits ACL checks for this kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub super_permission: Option<String>,

    /// Human-readable description of this resource kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Characters allowed in a CRD name (same as group IDs).
const CRD_NAME_ALLOWED_SPECIALS: &str = "_-";

impl CustomResourceDefinition {
    /// Validate the CRD name according to project naming rules:
    /// - 2-63 characters
    /// - Alphanumeric + `_` + `-`
    /// - Must not start with a digit or `-`
    /// - Must not be only digits
    /// - Lowercased
    pub fn validate_name(name: &str) -> Result<String, String> {
        let lowercased = name.to_lowercase();

        if lowercased.len() < 2 {
            return Err("CRD name must be at least 2 characters".into());
        }
        if lowercased.len() > 63 {
            return Err("CRD name must be at most 63 characters".into());
        }

        let first = lowercased.chars().next().unwrap();
        if first.is_ascii_digit() {
            return Err("CRD name cannot start with a digit".into());
        }
        if first == '-' {
            return Err("CRD name cannot start with '-'".into());
        }

        for c in lowercased.chars() {
            if !c.is_ascii_alphanumeric() && !CRD_NAME_ALLOWED_SPECIALS.contains(c) {
                return Err(format!(
                    "CRD name contains invalid character '{}'. Only alphanumerics, '_', and '-' are allowed",
                    c
                ));
            }
        }

        if lowercased.chars().all(|c| c.is_ascii_digit()) {
            return Err("CRD name cannot be only digits".into());
        }

        Ok(lowercased)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_crd_names() {
        assert_eq!(
            CustomResourceDefinition::validate_name("deployments").unwrap(),
            "deployments"
        );
        assert_eq!(
            CustomResourceDefinition::validate_name("My_Tasks").unwrap(),
            "my_tasks"
        );
        assert_eq!(
            CustomResourceDefinition::validate_name("ci-runs").unwrap(),
            "ci-runs"
        );
        assert_eq!(
            CustomResourceDefinition::validate_name("ab").unwrap(),
            "ab"
        );
    }

    #[test]
    fn invalid_crd_names() {
        // too short
        assert!(CustomResourceDefinition::validate_name("a").is_err());
        // starts with digit
        assert!(CustomResourceDefinition::validate_name("1tasks").is_err());
        // starts with hyphen
        assert!(CustomResourceDefinition::validate_name("-tasks").is_err());
        // only digits
        assert!(CustomResourceDefinition::validate_name("12345").is_err());
        // invalid chars
        assert!(CustomResourceDefinition::validate_name("my tasks").is_err());
        assert!(CustomResourceDefinition::validate_name("my.tasks").is_err());
    }

    #[test]
    fn field_type_serde_roundtrip() {
        let ft = FieldType::List {
            items: Box::new(FieldType::Enum {
                variants: vec!["active".into(), "paused".into(), "stopped".into()],
            }),
        };
        let json = serde_json::to_string(&ft).unwrap();
        let parsed: FieldType = serde_json::from_str(&json).unwrap();
        // Verify roundtrip by re-serializing and comparing JSON
        let json2 = serde_json::to_string(&parsed).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn full_crd_serde_roundtrip() {
        let mut fields = HashMap::new();
        fields.insert(
            "image".into(),
            FieldDef {
                field_type: FieldType::String,
                required: true,
                default: None,
                description: Some("Container image reference".into()),
                brief: true,
                immutable: false,
            },
        );
        fields.insert(
            "replicas".into(),
            FieldDef {
                field_type: FieldType::Int,
                required: false,
                default: Some(serde_json::json!(1)),
                description: None,
                brief: false,
                immutable: false,
            },
        );
        fields.insert(
            "status".into(),
            FieldDef {
                field_type: FieldType::Enum {
                    variants: vec!["pending".into(), "running".into(), "failed".into()],
                },
                required: false,
                default: Some(serde_json::json!("pending")),
                description: None,
                brief: true,
                immutable: false,
            },
        );

        let crd = CustomResourceDefinition {
            id: "deployments".into(),
            scope: CrdScope::Project,
            acl_mode: CrdAclMode::Inherit,
            nouns: CrdNouns {
                singular: "deployment".into(),
                plural: "deployments".into(),
            },
            relations: vec![CrdRelation {
                edge_collection: "deploys_to".into(),
                direction: RelationDirection::Outbound,
                target_kind: "environments".into(),
                label: Some("targets environment".into()),
            }],
            fields,
            id_prefix: "dep_".into(),
            super_permission: None,
            description: Some("Container deployment managed by the platform".into()),
            ..Default::default()
        };

        let yaml = serde_json::to_string_pretty(&crd).unwrap();
        let parsed: CustomResourceDefinition = serde_json::from_str(&yaml).unwrap();
        assert_eq!(parsed.id, "deployments");
        assert_eq!(parsed.scope, CrdScope::Project);
        assert_eq!(parsed.acl_mode, CrdAclMode::Inherit);
        assert_eq!(parsed.relations.len(), 1);
        assert_eq!(parsed.fields.len(), 3);
        assert!(parsed.fields["image"].required);
        assert!(!parsed.fields["replicas"].required);
    }

    #[test]
    fn nested_object_field() {
        let mut inner_fields = HashMap::new();
        inner_fields.insert(
            "cpu".into(),
            FieldDef {
                field_type: FieldType::String,
                required: true,
                default: None,
                description: Some("CPU limit, e.g. '500m'".into()),
                brief: false,
                immutable: false,
            },
        );
        inner_fields.insert(
            "memory".into(),
            FieldDef {
                field_type: FieldType::String,
                required: true,
                default: None,
                description: Some("Memory limit, e.g. '256Mi'".into()),
                brief: false,
                immutable: false,
            },
        );

        let fd = FieldDef {
            field_type: FieldType::Object {
                fields: inner_fields,
            },
            required: false,
            default: None,
            description: Some("Resource limits".into()),
            brief: false,
            immutable: false,
        };

        let json = serde_json::to_string(&fd).unwrap();
        let parsed: FieldDef = serde_json::from_str(&json).unwrap();
        assert!(parsed.description.as_deref() == Some("Resource limits"));
    }

    #[test]
    fn bitflags_field() {
        let mut bit_positions = HashMap::new();
        bit_positions.insert("can_deploy".into(), 0u8);
        bit_positions.insert("can_rollback".into(), 1u8);
        bit_positions.insert("can_scale".into(), 2u8);

        let fd = FieldDef {
            field_type: FieldType::Bitflags { bit_positions },
            required: false,
            default: Some(serde_json::json!(0)),
            description: None,
            brief: false,
            immutable: false,
        };

        let json = serde_json::to_string(&fd).unwrap();
        let parsed: FieldDef = serde_json::from_str(&json).unwrap();
        if let FieldType::Bitflags { bit_positions } = &parsed.field_type {
            assert_eq!(bit_positions.len(), 3);
            assert_eq!(bit_positions["can_deploy"], 0);
            assert_eq!(1u64 << bit_positions["can_rollback"], 2u64); // bit value = 1 << position
        } else {
            panic!("Expected Bitflags variant");
        }
    }
}
