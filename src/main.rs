
use dotenv::dotenv;
use ftx::{options::Options, rest::{Rest, GetFutures, Future, GetHistoricalPrices}};
use chrono::prelude::*;
use clap::{Parser, Subcommand};
use prettytable::{row, cell, Table};
use rust_decimal_macros::dec;

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
    Tops {
        #[clap(default_value = "10")]
        count: usize,
    },
    Download {
        market_name: String,
        start_time: DateTime<Local>,
        end_time: Option<DateTime<Local>>
    }
}

#[tokio::main]
async fn main() {
    // Read '.env' file
    dotenv().ok();

    // Read parameters
    let args = Args::parse();

    // Create a FTX connector
    let ftx = Rest::new(Options::from_env());

    match args.command {
        Commands::Tops { count } => tops(ftx, count).await,
        Commands::Download { market_name, start_time, end_time } =>
            unimplemented!("download is not implemented yet")
    }
}

async fn tops(ftx: Rest, count: usize) {
    let mut futures = ftx.request(GetFutures {}).await.unwrap();
    futures.sort_by(|a, b|
        a.volume_usd24h.unwrap().partial_cmp(&b.volume_usd24h.unwrap()).unwrap()
    );

    // Create a pretty table
    let mut table = Table::new();
    table.add_row(row![
        "Rank",
        "Market Name",
        "Description",
        "Volume in 24 HRs",
        "Changes (last 1 HR)",
        "Changes (last 24 HRs)",
        "Changes (today)",
    ]);
    for (rank, future) in futures.iter().rev().take(count).enumerate() {
        table.add_row(row![
            format!("{}", rank + 1),
            future.name,
            future.description,
            format!("{:.0} USD", future.volume_usd24h.unwrap()),
            format!("{:.2}%", future.change1h.unwrap() * dec!(100)),
            format!("{:.2}%", future.change24h.unwrap() * dec!(100)),
            format!("{:.2}%", future.change_bod.unwrap() * dec!(100)),
        ]);
    }

    // Print the result
    table.printstd();
}

async fn download(ftx: Rest) {
    // TODO: Create a data structure to store the results

    // TODO: Issue requests to fetch historical prices

    // TODO: Save the results to a CSV file
}
