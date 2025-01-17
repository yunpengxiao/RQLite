use crate::parser::SqlStatement;

pub fn execute(sql_statement: SqlStatement) {
    println!("{:?}", sql_statement);
}