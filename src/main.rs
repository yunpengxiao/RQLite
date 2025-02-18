#![feature(backtrace_frames)]
#![feature(error_generic_member_access)]

mod database;
mod executor;
mod page;
mod parser;
mod table;
mod utils;

use anyhow::Result;
use database::Database;
use executor::Executor;
use parser::sql_query;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

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
        statement: Option<String>,
    },

    Web,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut file = File::open(&cli.path)?;
    let database = Database::from(&mut file)?;
    //println!("{:?}", database);

    match cli.command {
        Commands::DbInfo => {
            println!("database page size: {}", database.get_page_size());
            println!("database page count: {}", database.get_page_count());
        }
        Commands::Tables => {
            let table_names = database.get_table_names();
            for name in table_names {
                println!("{name}");
            }
        }
        Commands::Run { statement } => {
            if let Some(stem) = statement {
                let statement = sql_query(stem.as_str()).unwrap();
                let executor = Executor::from(database);
                executor.execute(statement.1);
            } else {
                println!("No SQL statement to run!");
            }
        }
        Commands::Web => {
            main1();
        }
    }

    Ok(())
}

#[tokio::main]
async fn main1() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let buf_reader = BufReader::new(&mut stream);
                let request_lines = buf_reader.lines().into_iter().next().unwrap().unwrap();
                println!("{}", request_lines);
                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
