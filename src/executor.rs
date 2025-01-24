use crate::{page::Database, parser::SqlStatement};

pub struct Executor {
    //pub head_page_reader: PageReader,
    pub database: Database,
}

impl Executor {
    pub fn from(database: Database) -> Self {
        Self {
            database,
        }
    }

    pub fn execute(&self, sql_statement: SqlStatement) {
        println!("{:?}", sql_statement);
        match sql_statement {
            SqlStatement::SELECT(select_cmd) => {
                let table_name = select_cmd.table;
                //self.database.print_table_columns(table_name.as_str());
                let fields = select_cmd.fields;
                if fields.len() == 1 && fields[0] == "COUNT" {
                    println!("{}", self.database.count_rows(table_name.as_str()));
                } else {
                    for col in fields {
                        println!("{}: {:?}", 
                            col, self.database.get_column(table_name.as_str(), col.as_str()));
                    }
                }
            }
            SqlStatement::CREATE(creation_cmd) => {
                println!("This is a create cmd, doing nothing for now!");
            }
            _ => {
                println!("Not implement yet");
            }
        }
    }
}
