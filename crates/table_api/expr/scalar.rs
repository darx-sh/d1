/// A [`ScalarExpr`] computes a scalar value in SQL.
#[derive(Debug, Clone)]
pub enum ScalarExpr {
    /// A column reference
    Column(String),
}
