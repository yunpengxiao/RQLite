#![feature(backtrace_frames)]
#![feature(error_generic_member_access)]

mod page;
mod utils;

use anyhow::Result;
use page::{FileHeader, PageHeader, PageReader, RowReader};
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

    match &cli.command {
        Commands::DbInfo => {
            let mut file = File::open(&cli.path)?;
            let file_header = FileHeader::from(&mut file)?;
            let page_header = PageHeader::from(&mut file)?;
    
            println!("database page size: {}", file_header.page_size);
            println!("number of tables: {}", page_header.cell_count);
            println!("file header: {:?}", file_header);
            println!("page header: {:?}", page_header);
        },
        Commands::Tables => {
            let mut file = File::open(&cli.path)?;
            let row_reader = RowReader::from(&mut file)?;
            let num_of_cell = row_reader.pointers.len();
            for n in 0..num_of_cell {
                println!("table{n}'s name is {}", row_reader.read(n as u32)[1]);
            }
        },
        Commands::Run { statement } => {
            let mut file = File::open(&cli.path)?;
            if let Some(_stem) = statement {
                let _page_reader = PageReader::from(&mut file, 1);
            } else {
                println!("No SQL statement to run!");
            }
        }
    }

    Ok(())
}
