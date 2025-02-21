use crate::parser::{SqlStatement, sql_query};

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub table_name: String,
    pub cols: Vec<String>,
}

impl TableSchema {
    pub fn from(sql: &String) -> Self {
        let (_, sql_cmd) = sql_query(sql).unwrap();
        let mut table_name: String = String::new();
        let mut cols: Vec<String> = Vec::new();
        match sql_cmd {
            SqlStatement::CREATE(cs) => {
                table_name = cs.table_name;
                cols = cs.cols;
            }
            _ => println!("Something is wrong, the schema is not a creation sql."),
        }
        Self { table_name, cols }
    }
}
