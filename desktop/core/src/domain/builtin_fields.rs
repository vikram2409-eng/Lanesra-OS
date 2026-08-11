//! ADM-BR/ADM-WF "any field" targeting: a static registry of the built-in
//! (non-custom-field) columns each entity type exposes as a business-rule
//! condition/action or workflow trigger/action target, alongside the
//! already-fully-generic custom fields.
//!
//! Deliberately excludes three categories of column, none of which fit the
//! plain (key, string value) shape everything here assumes:
//! - Foreign keys (`company_id`, `contact_id`, `opportunity_id`, ...) - these
//!   need a relationship-aware record picker, which is what Custom
//!   Relationships (`relationship_service`) already solves; a generic text
//!   value input isn't the right UI for "the linked record is X".
//! - Generated/computed columns (`*_number`, document `subtotal_cents`/
//!   `total_cents`/..., `amount_paid_cents`/`balance_cents`) - either
//!   immutable identifiers or derived from line items/payments, so neither
//!   condition nor action makes sense against them.
//! - `owner_user_id` - already has a dedicated, purpose-built workflow
//!   action (`assign_owner`) and would need its own user-picker UI here;
//!   left out of the generic registry rather than half-supported.
//!
//! Each entity's status-equivalent field (`status`, `stage` for
//! Opportunity, `is_active` for Product) is registered as conditionable
//! but never actionable - it already has its own dedicated, better-suited
//! mechanism (the `status_changed` workflow trigger, and status transition
//! validation in each entity's own service), and letting a generic
//! `set_value` action silently overwrite it would sidestep that.

pub const BUILTIN_FIELD_TYPES: &[&str] = &["text", "number", "money", "percent", "date", "boolean", "select"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinField {
    pub key: &'static str,
    pub label: &'static str,
    pub field_type: &'static str,
    pub options: &'static [&'static str],
    /// Whether business-rule/workflow actions (require/hide/lock/
    /// set_default/set_value/update_field) may target this field, not just
    /// read it in a condition/trigger.
    pub actionable: bool,
}

const EMPTY_OPTIONS: &[&str] = &[];

macro_rules! field {
    ($key:expr, $label:expr, $type:expr, $actionable:expr) => {
        BuiltinField { key: $key, label: $label, field_type: $type, options: EMPTY_OPTIONS, actionable: $actionable }
    };
    ($key:expr, $label:expr, $type:expr, $options:expr, $actionable:expr) => {
        BuiltinField { key: $key, label: $label, field_type: $type, options: $options, actionable: $actionable }
    };
}

const COMPANY_FIELDS: &[BuiltinField] = &[
    field!("name", "Company name", "text", true),
    field!("status", "Status", "select", crate::models::company::COMPANY_STATUSES, false),
    field!("tax_number", "Tax number", "text", true),
    field!("billing_address", "Billing address", "text", true),
    field!("shipping_address", "Shipping address", "text", true),
    field!("tags", "Tags", "text", true),
    field!("notes", "Notes", "text", true),
];

const CONTACT_FIELDS: &[BuiltinField] = &[
    field!("first_name", "First name", "text", true),
    field!("last_name", "Last name", "text", true),
    field!("job_title", "Job title", "text", true),
    field!("email", "Email", "text", true),
    field!("phone", "Phone", "text", true),
    field!("mobile", "Mobile", "text", true),
    field!("is_primary", "Primary contact", "boolean", true),
    field!("status", "Status", "select", crate::models::contact::CONTACT_STATUSES, false),
    field!("tags", "Tags", "text", true),
    field!("notes", "Notes", "text", true),
];

const OPPORTUNITY_FIELDS: &[BuiltinField] = &[
    field!("name", "Opportunity name", "text", true),
    field!("stage", "Stage", "select", crate::models::opportunity::OPPORTUNITY_STAGES, false),
    field!("status", "Status", "select", crate::models::opportunity::OPPORTUNITY_STATUSES, false),
    field!("value", "Value", "money", true),
    field!("probability", "Probability", "percent", true),
    field!("expected_close_date", "Expected close date", "date", true),
    field!("lost_reason", "Lost reason", "text", true),
    field!("next_step", "Next step", "text", true),
];

const PRODUCT_FIELDS: &[BuiltinField] = &[
    field!("name", "Name", "text", true),
    field!("sku", "SKU", "text", true),
    field!("type", "Type", "select", crate::models::product::PRODUCT_TYPES, true),
    field!("category", "Category", "text", true),
    field!("description", "Description", "text", true),
    field!("unit_price", "Unit price", "money", true),
    field!("cost", "Cost", "money", true),
    field!("tax_rate", "Tax rate", "percent", true),
    field!("is_active", "Active", "boolean", false),
];

// Quote/Order/Invoice have no general-purpose `update()` in their services
// at all - by deliberate design (see quote_service.rs/order_service.rs/
// invoice_service.rs) these documents are immutable after creation except
// through status transitions (`set_status`) and conversion, matching real
// invoicing semantics: a sent quote's terms or an issued invoice's due date
// shouldn't silently change out from under the customer. So every field
// here is conditionable only - there is no safe write path for a generic
// action to route through without inventing document mutation that doesn't
// exist anywhere else in the product, which is well outside this registry's
// job to decide.
const QUOTE_FIELDS: &[BuiltinField] = &[
    field!("status", "Status", "select", crate::models::quote::QUOTE_STATUSES, false),
    field!("issue_date", "Issue date", "date", false),
    field!("expiry_date", "Valid until", "date", false),
    field!("terms", "Terms", "text", false),
    field!("notes", "Notes", "text", false),
];

const ORDER_FIELDS: &[BuiltinField] = &[
    field!("status", "Status", "select", crate::models::order::ORDER_STATUSES, false),
    field!("order_date", "Order date", "date", false),
    field!("notes", "Notes", "text", false),
];

const INVOICE_FIELDS: &[BuiltinField] = &[
    field!("status", "Status", "select", crate::models::invoice::INVOICE_STATUSES, false),
    field!("issue_date", "Issue date", "date", false),
    field!("due_date", "Due date", "date", false),
    field!("payment_terms", "Payment terms", "text", false),
    field!("notes", "Notes", "text", false),
];

const CONTRACT_FIELDS: &[BuiltinField] = &[
    field!("title", "Title", "text", true),
    field!("type", "Type", "text", true),
    field!("value", "Value", "money", true),
    field!("start_date", "Start date", "date", true),
    field!("end_date", "End date", "date", true),
    field!("renewal_date", "Renewal date", "date", true),
    field!("notice_period_days", "Notice period (days)", "number", true),
    field!("status", "Status", "select", crate::models::contract::CONTRACT_STATUSES, false),
    field!("notes", "Notes", "text", true),
];

const TASK_FIELDS: &[BuiltinField] = &[
    field!("title", "Title", "text", true),
    field!("description", "Description", "text", true),
    field!("priority", "Priority", "select", crate::models::task::TASK_PRIORITIES, true),
    field!("status", "Status", "select", crate::models::task::TASK_STATUSES, false),
    field!("due_date", "Due date", "date", true),
    field!("reminder_at", "Reminder", "date", true),
];

/// Every custom object shares this fixed built-in shape (see
/// `models::custom_record::CustomRecord`) regardless of which admin-defined
/// object it is - the object's own distinguishing fields are always custom
/// fields, never built-in ones.
const CUSTOM_RECORD_FIELDS: &[BuiltinField] = &[
    field!("primary_name", "Name", "text", true),
    field!("status", "Status", "select", crate::models::custom_object::CUSTOM_RECORD_STATUSES, false),
    field!("notes", "Notes", "text", true),
];

/// The built-in fields available as conditions/actions for `entity_type` -
/// one of the 9 core entities, or any active custom object (which all share
/// `CUSTOM_RECORD_FIELDS`, checked dynamically by the caller since this
/// function has no database access to confirm the object exists).
pub fn builtin_fields_for(entity_type: &str) -> &'static [BuiltinField] {
    match entity_type {
        "Company" => COMPANY_FIELDS,
        "Contact" => CONTACT_FIELDS,
        "Opportunity" => OPPORTUNITY_FIELDS,
        "Product" => PRODUCT_FIELDS,
        "Quote" => QUOTE_FIELDS,
        "Order" => ORDER_FIELDS,
        "Invoice" => INVOICE_FIELDS,
        "Contract" => CONTRACT_FIELDS,
        "Task" => TASK_FIELDS,
        _ => CUSTOM_RECORD_FIELDS,
    }
}

pub fn find_builtin_field(entity_type: &str, key: &str) -> Option<&'static BuiltinField> {
    builtin_fields_for(entity_type).iter().find(|f| f.key == key)
}

pub fn is_actionable_builtin_field(entity_type: &str, key: &str) -> bool {
    find_builtin_field(entity_type, key).is_some_and(|f| f.actionable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_entity_has_a_non_actionable_status_equivalent_field() {
        for entity in ["Company", "Contact", "Opportunity", "Quote", "Order", "Invoice", "Contract", "Task"] {
            let fields = builtin_fields_for(entity);
            assert!(
                fields.iter().any(|f| !f.actionable && f.field_type == "select"),
                "{entity} should have at least one non-actionable select (status-equivalent) field"
            );
        }
    }

    #[test]
    fn custom_objects_fall_back_to_the_shared_custom_record_shape() {
        assert_eq!(builtin_fields_for("vendor"), CUSTOM_RECORD_FIELDS);
        assert_eq!(builtin_fields_for("anything_unrecognized"), CUSTOM_RECORD_FIELDS);
    }

    #[test]
    fn find_and_actionability_helpers_agree_with_the_registry() {
        assert!(is_actionable_builtin_field("Company", "name"));
        assert!(!is_actionable_builtin_field("Company", "status"));
        assert!(find_builtin_field("Company", "not_a_real_field").is_none());
    }
}
