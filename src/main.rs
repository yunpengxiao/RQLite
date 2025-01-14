#![feature(backtrace_frames)]
#![feature(error_generic_member_access)]

mod page;
mod utils;

use anyhow::Result;
use page::PageReader;
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
    let first_page = PageReader::from(&mut file, 1);

    match &cli.command {
        Commands::DbInfo => {
            println!("database page size: {}", first_page.get_page_size());
            println!("number of tables: {}", first_page.table_count());
        },
        Commands::Tables => {
            let table_names = first_page.get_table_names();
            for name in table_names {
                println!("{name}");
            }
        },
        Commands::Run { statement } => {
            if let Some(_stem) = statement {
                let _page_reader = PageReader::from(&mut file, 1);
            } else {
                println!("No SQL statement to run!");
            }
        }
    }

    Ok(())
}
