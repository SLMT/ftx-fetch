use std::sync::mpsc::{channel, Receiver, Sender};

use chrono::prelude::*;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use ftx::{
    options::Options,
    rest::{Candle, Future, GetFutures, GetHistoricalPrices, Rest},
};
use prettytable::{cell, row, Table};
use rust_decimal_macros::dec;
use tokio::time::Instant;

type Ftx = Rest;

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
        #[clap(help = "name of the market")]
        market_name: String,
        #[clap(help = "start date of the history (YYYY-mm-dd)")]
        start_date: String,
        #[clap(help = "end date of the history (YYYY-mm-dd). Default: now")]
        end_date: Option<String>,
        #[clap(
            default_value = "15",
            help = "window length in seconds. \
            options: 15, 60, 300, 900, 3600, 14400, 86400, or \
            any multiple of 86400 up to 30*86400"
        )]
        resolution: u32,
    },
}

#[tokio::main]
async fn main() {
    // Read '.env' file
    dotenv().ok();

    // Read parameters
    let args = Args::parse();

    // Create a FTX connector
    let ftx = Ftx::new(Options::from_env());

    match args.command {
        Commands::Tops { count } => tops(ftx, count).await,
        Commands::Download {
            market_name,
            start_date,
            end_date,
            resolution,
        } => {
            let start_time: DateTime<Utc> = parse_date(&start_date).and_hms(0, 0, 0).into();
            let end_time: DateTime<Utc> = if let Some(end_date) = end_date {
                parse_date(&end_date).and_hms(23, 59, 59).into()
            } else {
                Utc::now()
            };

            dbg!(start_time);
            dbg!(end_time);
        }
    }
}

fn parse_date(date_str: &str) -> Date<Local> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
    Local.ymd(date.year(), date.month(), date.day())
}

async fn tops(ftx: Ftx, count: usize) {
    // Fetch futures and sort by volume (USD)
    let mut futures = ftx.request(GetFutures {}).await.unwrap();
    futures.sort_by(|a, b| {
        a.volume_usd24h
            .unwrap()
            .partial_cmp(&b.volume_usd24h.unwrap())
            .unwrap()
    });

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

    // Iterate futures reversely
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

async fn download(
    ftx: Ftx,
    market_name: String,
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    resolution: u32,
) {
    // Create a shared data structure to store the results
    let (tx, rx): (Sender<Vec<Candle>>, Receiver<Vec<Candle>>) = channel();

    // TODO: Issue requests to fetch historical prices
    let execute_time = Instant::now();
    let task = tokio::time::timeout_at(
        execute_time.clone(),
        fetch_historical_price(
            ftx,
            market_name,
            start_time,
            end_time,
            resolution,
            execute_time,
            tx,
        ),
    );
    tokio::spawn(task);

    // TODO: Save the results to a CSV file

    // unimplemented!();
}

async fn fetch_historical_price(
    ftx: Ftx,
    market_name: String,
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    resolution: u32,
    execute_time: Instant,
    out_channel: Sender<Vec<Candle>>,
) {
    let mut candles = ftx
        .request(GetHistoricalPrices {
            market_name,
            resolution,
            limit: None,
            start_time: Some(start_time.with_timezone(&Utc)),
            end_time: Some(end_time.with_timezone(&Utc)),
        })
        .await
        .unwrap();

    println!("{:?}", candles.first());
    println!("{:?}", candles.last());
}
