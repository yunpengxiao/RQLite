#![feature(backtrace_frames)]
#![feature(error_generic_member_access)]

mod page;
mod utils;
mod parser;
mod executor;

use anyhow::Result;
use executor::Executor;
use page::{Database, PageReader};
use parser::sql_query;
use std::fs::File;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// database path
    path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// list db info
    DbInfo,

    /// list tables
    Tables,

    Run {
        statement: Option<String>
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut file = File::open(&cli.path)?;
    let database = Database::from(&mut file)?;

    match cli.command {
        Commands::DbInfo => {
            println!("database page size: {}", database.get_page_size());
            println!("database page count: {}", database.get_page_count());
        },
        Commands::Tables => {
            let table_names = database.get_table_names();
            for name in table_names {
                println!("{name}");
            }
        },
        Commands::Run { statement } => {
            if let Some(stem) = statement {
                let statement = 
                    sql_query(stem.as_str()).unwrap();
                let executor = Executor::from(database);
                executor.execute(statement.1);
            } else {
                println!("No SQL statement to run!");
            }
        }
    }

    Ok(())
}
