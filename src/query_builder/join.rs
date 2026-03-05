//! Join and nested select support for PostgREST resource embedding.
//!
//! Enables structured join selection with inner/left join modifiers,
//! foreign key disambiguation, and aliases.

/// Modifier for join behavior (inner join filters parent rows; FK disambiguates multiple relationships).
#[derive(Debug, Clone)]
pub enum JoinModifier {
    /// Inner join: only return parent rows that have matching related rows.
    Inner,
    /// Explicit foreign key: use when multiple FKs exist between tables.
    ForeignKey(String),
}

/// Specification for a nested/joined resource in a select.
#[derive(Debug, Clone)]
pub struct JoinSpec {
    /// Relation (table) name.
    pub relation: String,
    /// Columns to select from the relation.
    pub columns: Vec<String>,
    /// Optional alias for the embedded resource.
    pub alias: Option<String>,
    /// Optional join modifier (inner, or FK name).
    pub modifier: Option<JoinModifier>,
}

impl JoinSpec {
    /// Create a new join spec for a relation with the given columns.
    pub fn new(relation: &str, columns: &[&str]) -> Self {
        Self {
            relation: relation.to_owned(),
            columns: columns.iter().map(|s| (*s).to_string()).collect(),
            alias: None,
            modifier: None,
        }
    }

    /// Set inner join modifier (filters parent rows when no match).
    pub fn inner(mut self) -> Self {
        self.modifier = Some(JoinModifier::Inner);
        self
    }

    /// Set explicit foreign key for disambiguation.
    pub fn foreign_key(mut self, fk_name: &str) -> Self {
        self.modifier = Some(JoinModifier::ForeignKey(fk_name.to_owned()));
        self
    }

    /// Set alias for the embedded resource.
    pub fn alias(mut self, alias: &str) -> Self {
        self.alias = Some(alias.to_owned());
        self
    }

    /// Render to PostgREST select fragment, e.g. `instruments!inner(id,name)` or `alias:relation(col1,col2)`.
    pub fn to_select_fragment(&self) -> String {
        let cols = self.columns.join(",");
        let relation_part = match &self.modifier {
            Some(JoinModifier::Inner) => format!("{}!inner", self.relation),
            Some(JoinModifier::ForeignKey(fk)) => format!("{}!{}", self.relation, fk),
            None => self.relation.clone(),
        };
        let fragment = format!("{}({})", relation_part, cols);
        if let Some(ref alias) = self.alias {
            format!("{}:{}", alias, fragment)
        } else {
            fragment
        }
    }
}
