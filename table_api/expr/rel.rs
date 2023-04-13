use crate::expr::scalar::ScalarExpr;
use std::sync::Arc;

/// A `RelationExpr` computes a SQL.
#[derive(Clone, Debug)]
pub enum RelationExpr {
    Table {
        name: String,
    },
    /// A projection supports basic column selection and relation selction.
    /// For relation selection, the [`ScalarExpr`] represents
    /// one-one, one-many, or many-many relationship between tables.
    /// For one-one and one-many relationship, the [`RelationExpr`] computes two SQL:
    ///   - the first SQL to select from current table
    ///   - the second SQL to select from the other table
    /// For many-many relationship, the [`RelationExpr`] computes one "join" SQL.
    Projection {
        exprs: Vec<ScalarExpr>,
        input: Arc<RelationExpr>,
    },
    Filter {
        predicate: ScalarExpr,
        input: Arc<RelationExpr>,
    },
    Values {
        values: Vec<ScalarExpr>,
    },
}

pub enum Statement {
    Select { rel_expr: RelationExpr },
}

struct RelBuilder {
    rel_expr: RelationExpr,
}

impl RelBuilder {
    pub fn table(name: &str) -> RelBuilder {
        RelBuilder {
            rel_expr: RelationExpr::Table {
                name: name.to_string(),
            },
        }
    }

    pub fn values(&self, values: Vec<ScalarExpr>) -> RelBuilder {
        RelBuilder {
            rel_expr: RelationExpr::Values { values },
        }
    }

    pub fn projection(&self, exprs: Vec<ScalarExpr>) -> RelBuilder {
        RelBuilder {
            rel_expr: RelationExpr::Projection {
                exprs,
                input: Arc::new(self.rel_expr.clone()),
            },
        }
    }

    pub fn filter(&self, predicate: ScalarExpr) -> RelBuilder {
        RelBuilder {
            rel_expr: RelationExpr::Filter {
                predicate,
                input: Arc::new(self.rel_expr.clone()),
            },
        }
    }
}
