
use dotenv::dotenv;
use ftx::{options::Options, rest::{Rest, GetFutures, Future, GetHistoricalPrices}};
use chrono::prelude::*;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = "FTX Price Fetcher")]
#[clap(author = "Yu-Shan Lin <sam123456777@gmail.com>")]
#[clap(version = "1.0")]
#[clap(about = "A tool to download price data from FTX.")]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Top10,
    Download {
        market_name: String,
        start_time: DateTime<Local>,
        end_time: Option<DateTime<Local>>
    }
}

fn main() {
    // Read '.env' file
    dotenv().ok();

    // Read parameters
    let args = Args::parse();

    println!("Hello, world!");

    // TODO: Get the parameters (symbol, start date, end date)

    // TODO: Create a data structure to store the results

    // TODO: Issue requests to fetch historical prices

    // TODO: Save the results to a CSV file
}


