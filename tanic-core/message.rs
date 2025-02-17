#[derive(Clone, Debug)]
pub struct NamespaceDeets {
    pub parts: Vec<String>,
    pub name: String,
    pub table_count: usize,
}

#[derive(Clone, Debug)]
pub struct TableDeets {
    pub namespace: Vec<String>,
    pub name: String,
    pub row_count: usize,
}

impl NamespaceDeets {
    pub fn from_parts(parts: Vec<String>) -> Self {
        let name = parts.clone().join(".");
        Self {
            parts,
            name,
            table_count: 0,
        }
    }
}
