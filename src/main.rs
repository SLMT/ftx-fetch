use chrono::prelude::*;
use clap::{Parser, Subcommand};
use csv::Writer;
use ftx::{
    options::Options,
    rest::{Candle, GetFutures, GetHistoricalPrices, Rest},
};
use log::{error, info};
use prettytable::{cell, row, Table};
use rust_decimal_macros::dec;
use tokio::time::{sleep_until, Instant};

type Ftx = Rest;
type StdDuration = std::time::Duration;
type ChDuration = chrono::Duration;

const REQUEST_INTERVAL: StdDuration = StdDuration::from_millis(20);

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

#[derive(thiserror::Error, Debug)]
enum FfError {
    #[error("FTX error: {0}")]
    Ftx(#[from] ftx::rest::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("Chrono parsing error: {0}")]
    ChronoParse(#[from] chrono::format::ParseError),
}

type FfResult<T> = Result<T, FfError>;

#[tokio::main]
async fn main() {
    // Init logging
    set_logger_level();
    pretty_env_logger::init();

    // Read parameters
    let args = Args::parse();

    // Create a FTX connector
    let ftx = Ftx::new(Options {
        endpoint: ftx::options::Endpoint::Com,
        key: None,
        secret: None,
        subaccount: None,
    });

    // Execute sub commands
    if let Err(error) = match args.command {
        Commands::Tops { count } => tops(ftx, count).await,
        Commands::Download {
            market_name,
            start_date,
            end_date,
            resolution,
        } => download(ftx, market_name, start_date, end_date, resolution).await,
    } {
        error!("{}", error.to_string());
    }
}

fn set_logger_level() {
    match std::env::var("RUST_LOG") {
        Ok(_) => {}
        Err(_) => std::env::set_var("RUST_LOG", "INFO"),
    }
}

fn parse_date(date_str: &str) -> FfResult<Date<Local>> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    Ok(Local.ymd(date.year(), date.month(), date.day()))
}

async fn tops(ftx: Ftx, count: usize) -> FfResult<()> {
    // Fetch futures and sort by volume (USD)
    let mut futures = ftx.request(GetFutures {}).await?;
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

    Ok(())
}

async fn download(
    ftx: Ftx,
    market_name: String,
    start_date: String,
    end_date: Option<String>,
    resolution: u32,
) -> FfResult<()> {
    let mut all_candles: Vec<Candle> = Vec::new();

    // Convert the date string to DateTime
    let start_time: DateTime<Local> = parse_date(&start_date)?.and_hms(0, 0, 0);
    let end_time: DateTime<Local> = if let Some(end_date) = end_date {
        parse_date(&end_date)?.and_hms(23, 59, 59)
    } else {
        Local::now()
    };

    // Issue requests to fetch historical prices
    let mut wakeup_time = Instant::now();
    let mut iter_count = 0;
    let mut next_end_time = end_time;
    loop {
        // Wait for a while to avoid reach rate limit
        sleep_until(wakeup_time).await;

        // Fetch the data
        let mut candles: Vec<Candle> = ftx
            .request(GetHistoricalPrices {
                market_name: market_name.clone(),
                resolution,
                limit: None,
                start_time: Some(start_time.with_timezone(&Utc)),
                end_time: Some(next_end_time.with_timezone(&Utc)),
            })
            .await?;

        // Stops when getting an empty result
        if candles.is_empty() {
            break;
        }

        // Prepare for the next request
        let current_start_time = candles.first().unwrap().start_time.with_timezone(&Local);
        next_end_time = current_start_time - ChDuration::seconds(1);
        wakeup_time += REQUEST_INTERVAL;

        // Save the result
        all_candles.append(&mut candles);

        // Report
        iter_count += 1;
        if iter_count % 10 == 0 {
            info!(
                "Data points from {} to {} ({} points) are downloaded.",
                current_start_time.format("%Y-%m-%d %H:%M:%S"),
                end_time.format("%Y-%m-%d %H:%M:%S"),
                all_candles.len()
            );
        }
    }

    info!(
        "Downloading finished. {} data points are downloaded.",
        all_candles.len()
    );

    // Sort the candles by time
    all_candles.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

    // Save the results to a CSV file
    let real_start_time = all_candles
        .first()
        .unwrap()
        .start_time
        .with_timezone(&Local);
    let real_end_time = all_candles.last().unwrap().start_time.with_timezone(&Local);
    let filename = format!(
        "{}-{}-{}.csv",
        market_name.to_lowercase(),
        real_start_time.format("%Y-%m-%d"),
        real_end_time.format("%Y-%m-%d")
    );
    info!("Saving the data to '{}'...", filename);
    save_to_csv(all_candles, &filename)?;

    info!("The data are saved to '{}'.", filename);

    Ok(())
}

fn save_to_csv(candles: Vec<Candle>, file_name: &str) -> csv::Result<()> {
    let mut writer = Writer::from_path(file_name)?;
    writer.write_record(&[
        "Start Time (Local)",
        "Open",
        "Close",
        "Low",
        "High",
        "Volume",
    ])?;

    for candle in candles {
        writer.write_record(&[
            candle
                .start_time
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            format!("{:.4}", candle.open),
            format!("{:.4}", candle.close),
            format!("{:.4}", candle.low),
            format!("{:.4}", candle.high),
            format!("{:.4}", candle.volume),
        ])?;
    }
    writer.flush()?;

    Ok(())
}
