# FTX Fetch

A command line tool to fetch historical prices of markets on [FTX](https://ftx.com/).

## Installation Guide

Step 1: Clone this repository.

```
> git clone https://github.com/SLMT/ftx-fetch.git
```

Step 2: Install `ftx-fetch` via `cargo` using the following command. If you don't have a Rust development environment, please follow the instructions [here](https://www.rust-lang.org/tools/install) to setup one.

```
> cd ftx-fetch
> cargo install --path .
```

Step 3: Check the installation

```
> ftx-fetch --help
```

## Usage

There two commands:

- `tops`: show the top 10 future markets that have the most trading in last 24 hours.
- `download`: download the historical prices of the specified market.

### Tops

```
> ftx-fetch tops [COUNT]
```

- `[COUNT]`: how many future markets to show. Default: 10

Example (some fields in the table are omitted):

```
> ftx-fetch tops 3

+------+-------------+----------------------------+------------------+-...-+
| Rank | Market Name | Description                | Volume in 24 HRs | ... |
+------+-------------+----------------------------+------------------+-...-+
| 1    | ETH-PERP    | Ethereum Perpetual Futures | 2365336295 USD   | ... |
+------+-------------+----------------------------+------------------+-...-+
| 2    | BTC-PERP    | Bitcoin Perpetual Futures  | 1971066791 USD   | ... |
+------+-------------+----------------------------+------------------+-...-+
| 3    | SOL-PERP    | Solana Perpetual Futures   | 274014478 USD    | ... |
+------+-------------+----------------------------+------------------+-...-+
```

### Download

```
> ftx-fetch download <MARKET_NAME> <START_DATE> [END_DATE] [RESOLUTION]
```

- `<MARKET_NAME>`: name of the market
- `<START_DATE>`: start date of the history (format: YYYY-mm-dd)
- `[END_DATE]`: end date of the history (format: YYYY-mm-dd). Default: now
- `[RESOLUTION]`: window length in seconds. options: 15, 60, 300, 900, 3600, 14400, 86400, or any multiple of 86400 up to 30*86400. Default: 15

Example:

```
> ftx-fetch download btc-perp 2022-07-20

INFO  ftx_fetch > Data points from 2022-07-22 07:56:00 to 2022-07-24 22:25:58 (15000 points) are downloaded.
INFO  ftx_fetch > Data points from 2022-07-19 17:26:00 to 2022-07-24 22:25:58 (30000 points) are downloaded.
INFO  ftx_fetch > Downloading finished. 39944 data points are downloaded.
INFO  ftx_fetch > Saving the data to 'btc-perp-2022-07-18-2022-07-24.csv'...
INFO  ftx_fetch > The data are saved to 'btc-perp-2022-07-18-2022-07-24.csv'.
```

It saves the result to a CSV file named with the market name, the start date and the end date.

## License

Copyright (c) 2022 Yu-Shan Lin.

ftx-fetch is made available under the terms of [the MIT License](LICENSE).
