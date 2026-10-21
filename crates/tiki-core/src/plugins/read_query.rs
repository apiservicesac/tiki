use serde::{Deserialize, Serialize};

/// One record returned by a client instance, as field name to value.
pub type Record = serde_json::Map<String, serde_json::Value>;

/// Comparison used by a query filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    In,
}

/// A single condition: `field operator value`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// A read-only query against a client instance. There is no write counterpart on purpose.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadQuery {
    /// Model or table, for example `hr.leave` in Odoo.
    pub model: String,
    pub fields: Vec<String>,
    pub filters: Vec<QueryFilter>,
    pub limit: usize,
}
