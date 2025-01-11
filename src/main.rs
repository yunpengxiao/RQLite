#![feature(backtrace_frames)]
#![feature(error_generic_member_access)]

mod page;
mod utils;

use anyhow::Result;
use page::{FileHeader, PageHeader, RowReader};
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
    DbInfo {
        table_name: Option<String>
    },

    /// list tables
    Tables,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::DbInfo { table_name } => {
            if let Some(_table_name) = table_name {
                let mut file = File::open(&cli.path)?;
                let _file_header = FileHeader::from(&mut file)?;
                let page_header = PageHeader::from(&mut file)?;
    
                //println!("database page size: {}", file_header.page_size);
                println!("number of tables: {}", page_header.cell_count);
            } else {
                let mut file = File::open(&cli.path)?;
                let file_header = FileHeader::from(&mut file)?;
                let page_header = PageHeader::from(&mut file)?;
    
                println!("database page size: {}", file_header.page_size);
                println!("number of tables: {}", page_header.cell_count);

                println!("file header: {:?}", file_header);
                println!("page header: {:?}", page_header);
            }
        },
        Commands::Tables => {
            let mut file = File::open(&cli.path)?;
            let num_of_cell = PageHeader::from(&mut file)?.cell_count;
            let row_reader = RowReader::from(&mut file, num_of_cell as usize)?;
            for n in 0..num_of_cell {
                println!("table{n}'s name is {}", row_reader.read(n.into())[1]);
            }
        }
    }

    Ok(())
}
