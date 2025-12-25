mod builder;
mod query;
mod table;

pub use builder::QueryBuilder;
pub use query::{paginate_query, CountResult};
pub use table::{paginate_by_id_range, paginate_table};

use paginator_rs::PaginatorError;

/// Validates that a field name is safe for use in SurrealQL queries.
/// Only allows alphanumeric characters, underscores, and dots (for qualified names).
/// Returns an error if the field name contains potentially dangerous characters.
pub fn validate_field_name(field: &str) -> Result<(), PaginatorError> {
    if field.is_empty() {
        return Err(PaginatorError::Custom(
            "Field name cannot be empty".to_string(),
        ));
    }

    for c in field.chars() {
        if !c.is_alphanumeric() && c != '_' && c != '.' {
            return Err(PaginatorError::Custom(format!(
                "Invalid field name '{}': contains unsafe character '{}'",
                field, c
            )));
        }
    }

    Ok(())
}
