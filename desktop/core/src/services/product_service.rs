use rusqlite::Connection;

use crate::domain::ids::new_uuid;
use crate::domain::numbering::{self, PRODUCT};
use crate::domain::{AppError, AppResult};
use crate::models::product::{Product, ProductInput, PRODUCT_TYPES};
use crate::repositories::{audit_repo, product_repo};

fn validate(input: &ProductInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Product name is required".into()));
    }
    if !PRODUCT_TYPES.contains(&input.r#type.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid product type '{}'",
            input.r#type
        )));
    }
    Ok(())
}

pub fn create(
    conn: &Connection,
    workspace_id: &str,
    input: &ProductInput,
    actor_user_id: Option<&str>,
) -> AppResult<Product> {
    validate(input)?;
    let id = new_uuid();
    let product_number = numbering::allocate_number(conn, workspace_id, &PRODUCT)?;
    let product = product_repo::create(conn, &id, workspace_id, &product_number, input, actor_user_id)?;
    audit_repo::record(
        conn,
        workspace_id,
        actor_user_id,
        "create",
        Some("product"),
        Some(&product.id),
        &format!("Created product {}", product.product_number),
        None,
    )?;
    Ok(product)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Product> {
    product_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Product".into()))
}

pub fn list(conn: &Connection, workspace_id: &str) -> AppResult<Vec<Product>> {
    Ok(product_repo::list(conn, workspace_id)?)
}

pub fn update(
    conn: &Connection,
    id: &str,
    input: &ProductInput,
    actor_user_id: Option<&str>,
) -> AppResult<Product> {
    validate(input)?;
    let workspace_id = get(conn, id)?.workspace_id;
    let product = product_repo::update(conn, id, input, actor_user_id)?;
    audit_repo::record(
        conn,
        &workspace_id,
        actor_user_id,
        "update",
        Some("product"),
        Some(id),
        &format!("Updated product {}", product.product_number),
        None,
    )?;
    Ok(product)
}

pub fn archive(conn: &Connection, id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    let existing = get(conn, id)?;
    product_repo::archive(conn, id, actor_user_id)?;
    audit_repo::record(
        conn,
        &existing.workspace_id,
        actor_user_id,
        "archive",
        Some("product"),
        Some(id),
        &format!("Archived product {}", existing.product_number),
        None,
    )?;
    Ok(())
}
