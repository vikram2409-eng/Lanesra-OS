//! Next-Gen program, Ontology Layer epic: the unified Object Type
//! registry (issue #341) and Link Type exposure (issue #342). No new
//! migration - a read model over `entity_registry::CORE_ENTITY_TYPES`,
//! `custom_object_service`, `relationship_service` and the Business
//! Glossary (FND-02), the same "don't invent a second mechanism" rule
//! every prior phase in this program has followed. Listing is open to
//! any authenticated user, matching `custom_object_service::list`'s own
//! "any authenticated user can list" precedent - this is a read view
//! over data those same open listings already expose.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::ontology::{LinkTypeSummary, ObjectTypeDetail, ObjectTypeSummary};
use crate::repositories::{glossary_repo, semantic_mapping_repo};
use crate::services::{custom_object_service, entity_registry, relationship_service};

/// `(key, singular label, plural label, icon)` - the plural labels and
/// icons already used throughout this codebase's own frontends
/// (`entityTypeLabel` in `types.ts`, the `icons` map in `app.js`),
/// consolidated here as the one authoritative source both frontends
/// should read from instead of each keeping (and risking drifting) its
/// own copy.
const BUILTIN_OBJECT_TYPES: &[(&str, &str, &str, &str)] = &[
    ("Company", "Company", "Companies", "◫"),
    ("Contact", "Contact", "Contacts", "◎"),
    ("Opportunity", "Opportunity", "Opportunities", "⌁"),
    ("Quote", "Quote", "Quotes", "▤"),
    ("Order", "Order", "Orders", "▣"),
    ("Invoice", "Invoice", "Invoices", "$"),
    ("Contract", "Contract", "Contracts", "▧"),
    ("Task", "Task", "Tasks", "✓"),
    ("Product", "Product", "Products", "◇"),
];

/// Deterministic and stable across calls (never randomized, never
/// persisted) - the same object type always maps to the same of the 6
/// categorical chart tokens within a workspace.
fn color_index_for(key: &str) -> i64 {
    let sum: u32 = key.bytes().map(u32::from).sum();
    (sum % 6) as i64 + 1
}

/// The `definition` of a whole-object Business Glossary mapping for
/// `object_type` (a `semantic_mapping` with `field_key: NULL` and a
/// `glossary_term_id` set) - `None` when no such mapping exists, never
/// a placeholder string.
fn description_for(conn: &Connection, workspace_id: &str, object_type: &str) -> AppResult<Option<String>> {
    let whole_object_term_id = semantic_mapping_repo::list_for_entity(conn, workspace_id, object_type)?
        .into_iter()
        .find(|m| m.field_key.is_none() && m.glossary_term_id.is_some())
        .and_then(|m| m.glossary_term_id);
    match whole_object_term_id {
        Some(term_id) => Ok(glossary_repo::get(conn, &term_id)?.map(|t| t.definition)),
        None => Ok(None),
    }
}

pub fn list_object_types(conn: &Connection, workspace_id: &str) -> AppResult<Vec<ObjectTypeSummary>> {
    let mut out = Vec::new();
    for (key, singular, plural, icon) in BUILTIN_OBJECT_TYPES {
        out.push(ObjectTypeSummary {
            key: key.to_string(),
            is_custom: false,
            label_singular: singular.to_string(),
            label_plural: plural.to_string(),
            icon: icon.to_string(),
            color_index: color_index_for(key),
            description: description_for(conn, workspace_id, key)?,
        });
    }
    for o in custom_object_service::list(conn, workspace_id, true)? {
        out.push(ObjectTypeSummary {
            key: o.key.clone(),
            is_custom: true,
            label_singular: o.singular_label,
            label_plural: o.plural_label,
            icon: o.icon,
            color_index: color_index_for(&o.key),
            description: description_for(conn, workspace_id, &o.key)?,
        });
    }
    Ok(out)
}

/// Every Link Type `object_type` participates in, as source or target,
/// oriented so `name`/`inverse_name` read naturally from `object_type`'s
/// own side - see `models::ontology::LinkTypeSummary`'s own doc comment.
fn link_types_for(conn: &Connection, workspace_id: &str, object_type: &str) -> AppResult<Vec<LinkTypeSummary>> {
    let mut out = Vec::new();
    for r in relationship_service::list(conn, workspace_id, true)? {
        if r.source_entity_type == object_type {
            out.push(LinkTypeSummary {
                relationship_id: r.id.clone(),
                name: r.forward_label.clone(),
                inverse_name: r.reverse_label.clone(),
                from_object_type: object_type.to_string(),
                to_object_type: if r.target_is_polymorphic { None } else { Some(r.target_entity_type.clone()) },
                relationship_type: r.relationship_type.clone(),
            });
        }
        if !r.target_is_polymorphic && r.target_entity_type == object_type && r.source_entity_type != object_type {
            out.push(LinkTypeSummary {
                relationship_id: r.id.clone(),
                name: r.reverse_label.clone(),
                inverse_name: r.forward_label.clone(),
                from_object_type: object_type.to_string(),
                to_object_type: Some(r.source_entity_type.clone()),
                relationship_type: r.relationship_type.clone(),
            });
        }
    }
    Ok(out)
}

pub fn get_object_type_detail(conn: &Connection, workspace_id: &str, object_type: &str) -> AppResult<ObjectTypeDetail> {
    let summary = list_object_types(conn, workspace_id)?
        .into_iter()
        .find(|o| o.key == object_type)
        .ok_or_else(|| AppError::NotFound("Object type".into()))?;
    let link_types = link_types_for(conn, workspace_id, object_type)?;
    Ok(ObjectTypeDetail { object_type: summary, link_types })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `BUILTIN_OBJECT_TYPES` is a separate, hand-maintained list (it
    /// carries a label/icon `entity_registry::CORE_ENTITY_TYPES` has no
    /// reason to know about) - this guards the one way the two could
    /// silently drift: a built-in entity type added to one and not the
    /// other.
    #[test]
    fn builtin_object_types_stay_in_sync_with_core_entity_types() {
        let registry_keys: Vec<&str> = BUILTIN_OBJECT_TYPES.iter().map(|(k, ..)| *k).collect();
        assert_eq!(registry_keys, entity_registry::CORE_ENTITY_TYPES);
    }

    #[test]
    fn color_index_is_deterministic_and_in_range() {
        for (key, ..) in BUILTIN_OBJECT_TYPES {
            let i = color_index_for(key);
            assert!((1..=6).contains(&i));
            assert_eq!(i, color_index_for(key), "must be stable across repeated calls");
        }
    }
}
