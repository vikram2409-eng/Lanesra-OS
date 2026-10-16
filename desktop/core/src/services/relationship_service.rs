//! Admin extensibility Phase B (spec §20.3/§21): lets an Administrator
//! define a relationship between any two entity types - built-in or
//! custom object - and lets any user link/unlink actual records through
//! it. Related-list rendering for any record is one function
//! (`related_records_for`) away regardless of how many relationship
//! definitions exist, matching the "custom relationships automatically
//! become eligible for related lists without requiring application code
//! changes" requirement (spec Platform Extensibility §7).

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::relationship::{
    RelatedRecord, RelationshipDefinition, RelationshipDefinitionInput, RelationshipDefinitionUpdate, RelationshipInstance,
    DELETE_BEHAVIORS, RELATIONSHIP_TYPES,
};
use crate::repositories::{audit_repo, custom_field_repo, relationship_repo};
use crate::services::{
    access_service, contact_service, contract_service, custom_field_service, custom_object_service, entity_registry, invoice_service,
    opportunity_service, order_service, quote_service, task_service,
};

/// Administrator always passes (unchanged); a non-Administrator additionally
/// passes with an explicit Access Role grant on "Relationship" - see
/// `access_service::require_admin_or_explicit_update`'s own doc comment.
fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    access_service::require_admin_or_explicit_update(conn, actor_user_id, "Relationship", "Only an Administrator can manage relationships")
}

/// FND-01 System Graph: a relationship depends on its source entity type
/// always, and its target entity type unless the target is polymorphic
/// (migration 0049) - there's no single fixed type to point an edge at
/// in that case, so this v1 slice leaves the target side unsynced rather
/// than guessing one.
fn sync_graph_node(conn: &Connection, workspace_id: &str, def: &RelationshipDefinition) -> AppResult<()> {
    let mut edges = vec![crate::models::system_graph::SystemEdgeTarget {
        edge_type: "depends_on".into(),
        to_node_type: "custom_object".into(),
        to_component_id: def.source_entity_type.clone(),
    }];
    if !def.target_is_polymorphic {
        edges.push(crate::models::system_graph::SystemEdgeTarget {
            edge_type: "depends_on".into(),
            to_node_type: "custom_object".into(),
            to_component_id: def.target_entity_type.clone(),
        });
    }
    super::system_graph_service::sync_node(conn, workspace_id, "relationship", &def.id, &def.forward_label, "{}", &edges)
}

fn require_valid_entity_type(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<()> {
    if entity_registry::CORE_ENTITY_TYPES.contains(&entity_type) {
        return Ok(());
    }
    if custom_object_service::is_valid_dynamic_entity_type(conn, workspace_id, entity_type)? {
        return Ok(());
    }
    Err(AppError::Validation(format!("'{entity_type}' is not a recognized object type")))
}

fn slugify(conn: &Connection, workspace_id: &str, source: &str, target: &str) -> AppResult<String> {
    let base = format!("{source}_{target}").to_lowercase();
    let existing = relationship_repo::list_definitions(conn, workspace_id)?;
    let existing_keys: Vec<&str> = existing.iter().map(|d| d.key.as_str()).collect();
    if !existing_keys.contains(&base.as_str()) {
        return Ok(base);
    }
    let mut suffix = 2;
    loop {
        let candidate = format!("{base}_{suffix}");
        if !existing_keys.contains(&candidate.as_str()) {
            return Ok(candidate);
        }
        suffix += 1;
    }
}

fn validate_shape(conn: &Connection, workspace_id: &str, input: &RelationshipDefinitionInput) -> AppResult<()> {
    require_valid_entity_type(conn, workspace_id, &input.source_entity_type)?;
    // A polymorphic-target definition doesn't fix one target type at
    // authoring time - `target_entity_type` is an unused placeholder (see
    // migration 0049), so there's nothing to validate against the
    // workspace's entity types here; `link()` validates the *chosen* type
    // per link instead.
    if !input.target_is_polymorphic {
        require_valid_entity_type(conn, workspace_id, &input.target_entity_type)?;
    }
    // Self-referential relationships (source == target - a parent/child
    // hierarchy: an Organization Unit's parent unit, a category tree, an
    // Asset's parent Asset) are allowed. `related_records_for` below was
    // already written to handle this shape correctly (see its own doc
    // comment); `link()` below guards the one thing that shape actually
    // needs and didn't have before: a record can't link to itself.
    if !RELATIONSHIP_TYPES.contains(&input.relationship_type.as_str()) {
        return Err(AppError::Validation(format!("Invalid relationship type '{}'", input.relationship_type)));
    }
    if !DELETE_BEHAVIORS.contains(&input.delete_behavior.as_str()) {
        return Err(AppError::Validation(format!("Invalid delete behavior '{}'", input.delete_behavior)));
    }
    if input.forward_label.trim().is_empty() || input.reverse_label.trim().is_empty() {
        return Err(AppError::Validation("Both direction labels are required".into()));
    }
    Ok(())
}

/// Builds the pickable field(s) the relationship itself promises ("build a
/// relationship and a lookup field appears on the right object(s)"),
/// instead of leaving the relationship only ever reachable as a
/// post-creation Related list. A single lookup field can only ever mean
/// "exactly one specific other record", so this only fires for the two
/// shapes that guarantee that: `many_to_one` (one field, on the "many"/
/// source side - e.g. Policy gets a "Customer" field) and `one_to_one`
/// (genuinely symmetric - one field on each side). `many_to_many` and a
/// polymorphic target are deliberately skipped - "many" can't collapse
/// into one field, and a polymorphic target has no single fixed type for
/// the field to point at (see this definition's own `sync_graph_node` doc
/// comment for the same reasoning). Not a hard requirement: if either
/// side's field creation fails (e.g. a key collision this relationship
/// didn't anticipate), the relationship itself still exists and is usable
/// via Related lists - the error is surfaced, not swallowed, but it never
/// un-does the relationship that was just created.
fn auto_create_lookup_fields(conn: &Connection, workspace_id: &str, def: &RelationshipDefinition, actor_user_id: Option<&str>) -> AppResult<()> {
    if def.target_is_polymorphic {
        return Ok(());
    }
    match def.relationship_type.as_str() {
        "many_to_one" => {
            custom_field_service::create_lookup_field(
                conn, workspace_id, &def.source_entity_type, &def.forward_label, &def.target_entity_type, &def.id, actor_user_id,
            )?;
        }
        "one_to_one" => {
            custom_field_service::create_lookup_field(
                conn, workspace_id, &def.source_entity_type, &def.forward_label, &def.target_entity_type, &def.id, actor_user_id,
            )?;
            custom_field_service::create_lookup_field(
                conn, workspace_id, &def.target_entity_type, &def.reverse_label, &def.source_entity_type, &def.id, actor_user_id,
            )?;
        }
        _ => {} // many_to_many: no single-record field makes sense
    }
    Ok(())
}

pub fn create(conn: &Connection, workspace_id: &str, input: &RelationshipDefinitionInput, actor_user_id: Option<&str>) -> AppResult<RelationshipDefinition> {
    require_admin(conn, actor_user_id)?;
    validate_shape(conn, workspace_id, input)?;
    let target_for_key = if input.target_is_polymorphic { "any" } else { input.target_entity_type.as_str() };
    let key = slugify(conn, workspace_id, &input.source_entity_type, target_for_key)?;
    let id = crate::domain::ids::new_uuid();
    let created = relationship_repo::create_definition(conn, &id, workspace_id, &key, input, actor_user_id)?;
    super::solution_component_service::tag_local(conn, workspace_id, "relationship_definition", &created.id, actor_user_id)?;
    sync_graph_node(conn, workspace_id, &created)?;
    auto_create_lookup_fields(conn, workspace_id, &created, actor_user_id)?;
    audit_repo::record(conn, workspace_id, actor_user_id, "create", Some("relationship"), Some(&created.id), &format!("Created relationship '{}'", created.forward_label), None)?;
    Ok(created)
}

pub fn list(conn: &Connection, workspace_id: &str, active_only: bool) -> AppResult<Vec<RelationshipDefinition>> {
    let all = relationship_repo::list_definitions(conn, workspace_id)?;
    Ok(if active_only { all.into_iter().filter(|d| d.is_active).collect() } else { all })
}

/// See `custom_object_service::get`'s doc comment - same export use case.
pub fn get(conn: &Connection, id: &str) -> AppResult<Option<RelationshipDefinition>> {
    Ok(relationship_repo::get_definition(conn, id)?)
}

pub fn update(conn: &Connection, id: &str, input: &RelationshipDefinitionUpdate, actor_user_id: Option<&str>) -> AppResult<RelationshipDefinition> {
    require_admin(conn, actor_user_id)?;
    let existing = relationship_repo::get_definition(conn, id)?.ok_or_else(|| AppError::NotFound("Relationship".into()))?;
    if existing.is_protected {
        return Err(AppError::Validation("This relationship is protected by the system and cannot be modified".into()));
    }
    if !DELETE_BEHAVIORS.contains(&input.delete_behavior.as_str()) {
        return Err(AppError::Validation(format!("Invalid delete behavior '{}'", input.delete_behavior)));
    }
    if input.forward_label.trim().is_empty() || input.reverse_label.trim().is_empty() {
        return Err(AppError::Validation("Both direction labels are required".into()));
    }
    let updated = relationship_repo::update_definition(conn, id, input, actor_user_id)?;
    sync_graph_node(conn, &existing.workspace_id, &updated)?;
    // Deactivating a relationship deactivates its auto-created Lookup
    // field(s) too (soft, same "deactivate never hard-delete a
    // referenceable definition" rule every other definition here follows)
    // - re-activating the relationship does NOT re-activate the field(s),
    // since the admin may have deliberately removed it from a form since;
    // creating a brand new relationship is how a lookup field comes back.
    if existing.is_active && !updated.is_active {
        custom_field_service::deactivate_lookup_fields_for_relationship(conn, id, actor_user_id)?;
    }
    audit_repo::record(conn, &existing.workspace_id, actor_user_id, "update", Some("relationship"), Some(id), &format!("Updated relationship '{}'", input.forward_label), None)?;
    Ok(updated)
}

/// Hard-deletes a definition. Blocked while any link through it still
/// exists, mirroring custom_object_service::delete's "archive/deactivate
/// instead, or clear it out first" rule.
pub fn delete(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let existing = relationship_repo::get_definition(conn, id)?.ok_or_else(|| AppError::NotFound("Relationship".into()))?;
    if existing.is_protected {
        return Err(AppError::Validation("This relationship is protected by the system and cannot be deleted".into()));
    }
    let count = relationship_repo::count_instances_for_definition(conn, id)?;
    if count > 0 {
        return Err(AppError::Validation(format!(
            "Cannot delete this relationship - {count} record(s) are still linked through it. Unlink them first, or deactivate the relationship instead."
        )));
    }
    // Deactivate any auto-created Lookup field(s) and clear their
    // back-reference *before* the hard delete below -
    // `lookup_relationship_definition_id` is a foreign key against this
    // row, so the delete would otherwise fail with those fields still
    // pointing at it.
    custom_field_service::detach_lookup_fields_for_deleted_relationship(conn, id, actor_user_id)?;
    relationship_repo::delete_definition(conn, id)?;
    super::system_graph_service::remove_node(conn, &existing.workspace_id, "relationship", id)?;
    audit_repo::record(conn, &existing.workspace_id, actor_user_id, "delete", Some("relationship"), Some(id), &format!("Deleted relationship '{}'", existing.forward_label), None)?;
    Ok(())
}

fn resolve_or_error(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<entity_registry::ResolvedRecord> {
    entity_registry::resolve(conn, entity_type, entity_id)?
        .ok_or_else(|| AppError::Validation(format!("{entity_type} record does not exist")))
}

/// Links two records through `definition_id`. `source_entity_type`/
/// `target_entity_type` must match the definition's own orientation
/// exactly - the frontend always knows which side it's calling from, so
/// this stays a direct, unambiguous write rather than guessing direction.
pub fn link(
    conn: &Connection,
    workspace_id: &str,
    definition_id: &str,
    source_entity_type: &str,
    source_id: &str,
    target_entity_type: &str,
    target_id: &str,
    actor_user_id: Option<&str>,
) -> AppResult<RelationshipInstance> {
    let def = relationship_repo::get_definition(conn, definition_id)?.ok_or_else(|| AppError::NotFound("Relationship".into()))?;
    if !def.is_active {
        return Err(AppError::Validation("This relationship is not active".into()));
    }
    if def.source_entity_type != source_entity_type {
        return Err(AppError::Validation("Record types do not match this relationship's definition".into()));
    }
    if def.target_is_polymorphic {
        // Any real object type in the workspace is a valid target, chosen
        // per link rather than fixed on the definition.
        require_valid_entity_type(conn, workspace_id, target_entity_type)?;
    } else if def.target_entity_type != target_entity_type {
        return Err(AppError::Validation("Record types do not match this relationship's definition".into()));
    }
    // Only reachable at all for a self-referential definition (source and
    // target entity types equal) - a record can't link to itself.
    if source_entity_type == target_entity_type && source_id == target_id {
        return Err(AppError::Validation("A record cannot be linked to itself".into()));
    }

    let source = resolve_or_error(conn, source_entity_type, source_id)?;
    let target = resolve_or_error(conn, target_entity_type, target_id)?;
    if source.workspace_id != workspace_id || target.workspace_id != workspace_id {
        return Err(AppError::Validation("Both records must belong to this workspace".into()));
    }

    match def.relationship_type.as_str() {
        "many_to_one" => {
            if relationship_repo::count_by_source(conn, definition_id, source_id)? > 0 {
                return Err(AppError::Validation(format!(
                    "This {source_entity_type} record is already linked as {} - unlink it first to relink",
                    def.forward_label
                )));
            }
        }
        "one_to_one" => {
            if relationship_repo::count_by_source(conn, definition_id, source_id)? > 0
                || relationship_repo::count_by_target(conn, definition_id, target_id)? > 0
            {
                return Err(AppError::Validation("One or both records are already linked through this one-to-one relationship".into()));
            }
        }
        _ => {} // many_to_many: only the UNIQUE(definition, source, target) constraint applies
    }

    let id = crate::domain::ids::new_uuid();
    let instance = relationship_repo::create_instance(conn, &id, workspace_id, definition_id, source_entity_type, source_id, target_entity_type, target_id, actor_user_id)
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(err, _) if err.code == rusqlite::ErrorCode::ConstraintViolation => {
                AppError::Validation("These two records are already linked".into())
            }
            other => AppError::from(other),
        })?;
    sync_lookup_caches(conn, &def, source_id, target_id, true)?;
    Ok(instance)
}

pub fn unlink(conn: &Connection, instance_id: &str, _actor_user_id: Option<&str>) -> AppResult<()> {
    let inst = relationship_repo::get_instance(conn, instance_id)?.ok_or_else(|| AppError::NotFound("Link".into()))?;
    clear_lookup_caches_for_instance(conn, &inst)?;
    Ok(relationship_repo::delete_instance(conn, instance_id)?)
}

/// Keeps every Lookup field's `value_text` cache in step with the
/// relationship_instances row it mirrors, whichever path made or removed
/// the link - a lookup field save, a workflow's `create_record`, the
/// Related-list UI, an import or the API. Without this a link made by
/// anything other than a lookup field save left the field's cache empty,
/// and the next ordinary save of that record wrote the empty value back
/// through `set_lookup_value`, silently deleting the link.
///
/// A field on the definition's source type holds the target's id; the
/// reverse field a one_to_one definition also creates on the target type
/// holds the source's id - the same side rule `set_lookup_value` uses.
/// On unlink a cache is cleared only if it still points at this link's
/// other record, so it never wipes a value that already moved on.
fn sync_lookup_caches(conn: &Connection, def: &RelationshipDefinition, source_id: &str, target_id: &str, linked: bool) -> AppResult<()> {
    for field in custom_field_repo::find_by_lookup_relationship(conn, &def.id)? {
        let (record_id, other_id) = if field.entity_type == def.source_entity_type { (source_id, target_id) } else { (target_id, source_id) };
        if linked {
            custom_field_repo::set_value(conn, &field.id, record_id, other_id)?;
        } else {
            let cached = custom_field_repo::get_values(conn, record_id)?.remove(&field.key);
            if cached.as_deref() == Some(other_id) {
                custom_field_repo::set_value(conn, &field.id, record_id, "")?;
            }
        }
    }
    Ok(())
}

fn clear_lookup_caches_for_instance(conn: &Connection, inst: &RelationshipInstance) -> AppResult<()> {
    if let Some(def) = relationship_repo::get_definition(conn, &inst.relationship_definition_id)? {
        sync_lookup_caches(conn, &def, &inst.source_id, &inst.target_id, false)?;
    }
    Ok(())
}

/// Writes a Lookup Custom Field's value as a real relationship link -
/// called from `custom_field_service::set_entity_values` whenever a
/// `field_type == "lookup"` field is saved, so `related_records_for`,
/// Object 360, and the Related-list UI all see the exact same link a
/// lookup field shows, never a second shadow state that can drift from
/// it (the field's own `value_text` cache is written separately, by the
/// caller, from the same value this function is given).
///
/// `field_entity_type`/`field_entity_id` identify the record the lookup
/// field lives on, which is either this relationship's source or target
/// side (both are possible - see `auto_create_lookup_fields`'
/// `one_to_one` branch, which creates a field on each side of the same
/// definition). `new_value` is the id of the record to link to; `None` or
/// empty clears whatever link currently exists for this side.
pub fn set_lookup_value(
    conn: &Connection,
    workspace_id: &str,
    relationship_definition_id: &str,
    field_entity_type: &str,
    field_entity_id: &str,
    new_value: Option<&str>,
    actor_user_id: Option<&str>,
) -> AppResult<()> {
    let def = relationship_repo::get_definition(conn, relationship_definition_id)?
        .ok_or_else(|| AppError::NotFound("Relationship".into()))?;
    let field_is_source = def.source_entity_type == field_entity_type;
    let other_entity_type = if field_is_source { def.target_entity_type.clone() } else { def.source_entity_type.clone() };

    let existing_instance = if field_is_source {
        relationship_repo::list_instances_where_source(conn, &def.id, field_entity_id)?.into_iter().next()
    } else {
        relationship_repo::list_instances_where_target(conn, &def.id, field_entity_id)?.into_iter().next()
    };

    let new_target_id = match new_value.map(|v| v.trim()).filter(|v| !v.is_empty()) {
        Some(v) => v,
        None => {
            if let Some(inst) = existing_instance {
                unlink(conn, &inst.id, actor_user_id)?;
            }
            return Ok(());
        }
    };

    if let Some(inst) = &existing_instance {
        let currently_linked_to = if field_is_source { inst.target_id.as_str() } else { inst.source_id.as_str() };
        if currently_linked_to == new_target_id {
            return Ok(()); // already linked to this record - no-op
        }
    }
    if let Some(inst) = existing_instance {
        unlink(conn, &inst.id, actor_user_id)?;
    }

    if field_is_source {
        link(conn, workspace_id, &def.id, field_entity_type, field_entity_id, &other_entity_type, new_target_id, actor_user_id)?;
    } else {
        link(conn, workspace_id, &def.id, &other_entity_type, new_target_id, field_entity_type, field_entity_id, actor_user_id)?;
    }
    Ok(())
}

/// Every related record for `entity_type`/`entity_id`, across every active
/// relationship it participates in from either direction - what a detail
/// page's "related lists" section renders. `show_related_list = 0`
/// definitions are still linkable but omitted here, since that flag exists
/// specifically to keep an internal-only relationship out of this view.
pub fn related_records_for(conn: &Connection, workspace_id: &str, entity_type: &str, entity_id: &str) -> AppResult<Vec<RelatedRecord>> {
    let defs = relationship_repo::list_definitions_for_entity(conn, workspace_id, entity_type)?;
    let mut out = Vec::new();

    for def in defs.iter().filter(|d| d.show_related_list) {
        if def.source_entity_type == entity_type {
            for inst in relationship_repo::list_instances_where_source(conn, &def.id, entity_id)? {
                if let Some(other) = entity_registry::resolve(conn, &inst.target_entity_type, &inst.target_id)? {
                    out.push(RelatedRecord {
                        instance_id: inst.id,
                        relationship_definition_id: def.id.clone(),
                        relationship_key: def.key.clone(),
                        label: def.forward_label.clone(),
                        entity_type: inst.target_entity_type,
                        entity_id: inst.target_id,
                        display_name: other.display_name,
                        status: other.status,
                        archived: other.archived,
                    });
                }
            }
        }
        // Not `else if` - a self-referencing-shaped pair of custom objects
        // could theoretically match both; in practice source != target is
        // enforced at creation, so at most one branch ever contributes rows
        // for a given definition. For a polymorphic-target definition,
        // `def.target_entity_type` is the unused '' placeholder (never
        // equal to a real entity_type), so this branch is instead gated by
        // `target_is_polymorphic` and each instance's *own* stored type is
        // checked to confirm this record is actually a target of it -
        // `list_definitions_for_entity` surfaces every polymorphic
        // definition regardless of the viewed type, precisely because the
        // definition itself can't say which types are eligible.
        if def.target_entity_type == entity_type || def.target_is_polymorphic {
            for inst in relationship_repo::list_instances_where_target(conn, &def.id, entity_id)? {
                if def.target_is_polymorphic && inst.target_entity_type != entity_type {
                    continue;
                }
                if let Some(other) = entity_registry::resolve(conn, &inst.source_entity_type, &inst.source_id)? {
                    out.push(RelatedRecord {
                        instance_id: inst.id,
                        relationship_definition_id: def.id.clone(),
                        relationship_key: def.key.clone(),
                        label: def.reverse_label.clone(),
                        entity_type: inst.source_entity_type,
                        entity_id: inst.source_id,
                        display_name: other.display_name,
                        status: other.status,
                        archived: other.archived,
                    });
                }
            }
        }
    }

    Ok(out)
}

/// Object 360 (issue #352): the built-in CRM's own FK-based relationships
/// (`company_id` on contacts/opportunities/quotes/orders/invoices/
/// contracts, `Task.related_type`/`related_id`, `Quote.opportunity_id`,
/// `Order.source_quote_id`, `Invoice.source_order_id`, ...) completely
/// bypass `relationship_definitions`/`relationship_instances` above - they
/// predate that generic, admin-authored system and were never migrated
/// onto it. `related_records_for` alone would show a fresh workspace's
/// own Company/Contact/etc. records as having no related records at all,
/// since nothing there is admin-defined. This is the same enumerated set
/// of pairs every existing detail page already hand-queries for its own
/// "Related" cards (see `Companies.tsx`'s own `listContactsByCompany`/
/// `listOpportunitiesByCompany`/... calls) - consolidated once here
/// instead of re-deriving per caller. Object 360 unions this with
/// `related_records_for` above rather than one replacing the other.
pub fn native_related_records_for(conn: &Connection, workspace_id: &str, entity_type: &str, entity_id: &str) -> AppResult<Vec<RelatedRecord>> {
    let mut out = Vec::new();
    let push = |conn: &Connection, out: &mut Vec<RelatedRecord>, target_type: &str, target_id: &str, group_label: &str, relationship_key: &str| -> AppResult<()> {
        if let Some(other) = entity_registry::resolve(conn, target_type, target_id)? {
            out.push(RelatedRecord {
                instance_id: target_id.to_string(),
                relationship_definition_id: String::new(),
                relationship_key: relationship_key.to_string(),
                label: group_label.to_string(),
                entity_type: target_type.to_string(),
                entity_id: target_id.to_string(),
                display_name: other.display_name,
                status: other.status,
                archived: other.archived,
            });
        }
        Ok(())
    };

    match entity_type {
        "Company" => {
            for c in contact_service::list_by_company(conn, entity_id)? { push(conn, &mut out, "Contact", &c.id, "Contacts", "company_contacts")?; }
            for o in opportunity_service::list_by_company(conn, entity_id)? { push(conn, &mut out, "Opportunity", &o.id, "Opportunities", "company_opportunities")?; }
            for q in quote_service::list(conn, workspace_id)?.into_iter().filter(|q| q.company_id == entity_id) { push(conn, &mut out, "Quote", &q.id, "Quotes", "company_quotes")?; }
            for o in order_service::list(conn, workspace_id)?.into_iter().filter(|o| o.company_id == entity_id) { push(conn, &mut out, "Order", &o.id, "Orders", "company_orders")?; }
            for i in invoice_service::list(conn, workspace_id)?.into_iter().filter(|i| i.company_id == entity_id) { push(conn, &mut out, "Invoice", &i.id, "Invoices", "company_invoices")?; }
            for c in contract_service::list_by_company(conn, entity_id)? { push(conn, &mut out, "Contract", &c.id, "Contracts", "company_contracts")?; }
            for t in task_service::list_by_related(conn, "Company", entity_id)? { push(conn, &mut out, "Task", &t.id, "Tasks", "company_tasks")?; }
        }
        "Contact" => {
            for o in opportunity_service::list(conn, workspace_id)?.into_iter().filter(|o| o.primary_contact_id.as_deref() == Some(entity_id)) { push(conn, &mut out, "Opportunity", &o.id, "Opportunities", "contact_opportunities")?; }
            for q in quote_service::list(conn, workspace_id)?.into_iter().filter(|q| q.contact_id.as_deref() == Some(entity_id)) { push(conn, &mut out, "Quote", &q.id, "Quotes", "contact_quotes")?; }
            for o in order_service::list(conn, workspace_id)?.into_iter().filter(|o| o.contact_id.as_deref() == Some(entity_id)) { push(conn, &mut out, "Order", &o.id, "Orders", "contact_orders")?; }
            for i in invoice_service::list(conn, workspace_id)?.into_iter().filter(|i| i.contact_id.as_deref() == Some(entity_id)) { push(conn, &mut out, "Invoice", &i.id, "Invoices", "contact_invoices")?; }
            for c in contract_service::list(conn, workspace_id)?.into_iter().filter(|c| c.contact_id.as_deref() == Some(entity_id)) { push(conn, &mut out, "Contract", &c.id, "Contracts", "contact_contracts")?; }
            for t in task_service::list_by_related(conn, "Contact", entity_id)? { push(conn, &mut out, "Task", &t.id, "Tasks", "contact_tasks")?; }
        }
        "Opportunity" => {
            for q in quote_service::list(conn, workspace_id)?.into_iter().filter(|q| q.opportunity_id.as_deref() == Some(entity_id)) { push(conn, &mut out, "Quote", &q.id, "Quotes", "opportunity_quotes")?; }
            for t in task_service::list_by_related(conn, "Opportunity", entity_id)? { push(conn, &mut out, "Task", &t.id, "Tasks", "opportunity_tasks")?; }
        }
        "Quote" => {
            for o in order_service::list(conn, workspace_id)?.into_iter().filter(|o| o.source_quote_id.as_deref() == Some(entity_id)) { push(conn, &mut out, "Order", &o.id, "Orders", "quote_orders")?; }
            for c in contract_service::list(conn, workspace_id)?.into_iter().filter(|c| c.source_quote_id.as_deref() == Some(entity_id)) { push(conn, &mut out, "Contract", &c.id, "Contracts", "quote_contracts")?; }
            for t in task_service::list_by_related(conn, "Quote", entity_id)? { push(conn, &mut out, "Task", &t.id, "Tasks", "quote_tasks")?; }
        }
        "Order" => {
            for i in invoice_service::list(conn, workspace_id)?.into_iter().filter(|i| i.source_order_id.as_deref() == Some(entity_id)) { push(conn, &mut out, "Invoice", &i.id, "Invoices", "order_invoices")?; }
            for t in task_service::list_by_related(conn, "Order", entity_id)? { push(conn, &mut out, "Task", &t.id, "Tasks", "order_tasks")?; }
        }
        "Invoice" => {
            for t in task_service::list_by_related(conn, "Invoice", entity_id)? { push(conn, &mut out, "Task", &t.id, "Tasks", "invoice_tasks")?; }
        }
        "Contract" => {
            for t in task_service::list_by_related(conn, "Contract", entity_id)? { push(conn, &mut out, "Task", &t.id, "Tasks", "contract_tasks")?; }
        }
        "Task" => {
            let task = task_service::get(conn, entity_id)?;
            if let (Some(related_type), Some(related_id)) = (task.related_type, task.related_id) {
                push(conn, &mut out, &related_type, &related_id, "Related to", "task_related_to")?;
            }
        }
        _ => {}
    }
    Ok(out)
}

/// Checked before a record is archived/deleted: any `restrict` relationship
/// still linking to it blocks the operation; any `archive` relationship
/// has its link rows silently cleared instead (ADM-CR-06 - "destructive
/// cascade is not the default", so the linked *other* record is never
/// touched, only the link itself).
pub fn enforce_delete_behavior(conn: &Connection, entity_type: &str, entity_id: &str) -> AppResult<()> {
    let instances = relationship_repo::list_instances_for_record(conn, entity_type, entity_id)?;
    if instances.is_empty() {
        return Ok(());
    }

    let mut to_clear = Vec::new();
    for inst in &instances {
        let def = relationship_repo::get_definition(conn, &inst.relationship_definition_id)?;
        let restrict = def.map(|d| d.delete_behavior == "restrict").unwrap_or(true);
        if restrict {
            return Err(AppError::Validation(
                "This record is still linked to other records through a custom relationship - unlink it first, or change the relationship's delete behavior to Archive".into(),
            ));
        }
        to_clear.push(inst.id.clone());
    }
    for id in to_clear {
        if let Some(inst) = instances.iter().find(|i| i.id == id) {
            clear_lookup_caches_for_instance(conn, inst)?;
        }
        relationship_repo::delete_instance(conn, &id)?;
    }
    Ok(())
}
