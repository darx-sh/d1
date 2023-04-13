use crate::catalog::schema::{ColumnDesc, Datum};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ColumnData {
    column: ColumnDesc,
    datum: Datum,
}

#[derive(Debug)]
pub struct Row {
    columns: Vec<ColumnData>,
}

impl Serialize for Row {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let map_data = self
            .columns
            .iter()
            .map(|d| (d.column.clone().name, d.datum.clone()))
            .collect::<HashMap<_, _>>();
        map_data.serialize(serializer)
    }
}
