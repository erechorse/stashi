# stashi

stashi is a CLI app that uses GMO Coin’s API to automate cryptocurrency accumulation with Dollar-Cost Averaging (DCA).

## Prerequisites

- You must have cargo installed.
- You must have [set up API access with GMO Coin](https://coin.z.com/jp/corp/product/info/api/).
- Ensure you have an environment capable of running commands regularly, regardless of the time of day (e.g., via cron).

## Installation

Install stashi using cargo:

```bash
$ cargo install --git https://github.com/erechorse/stashi.git
```

## Configuration

Create a config.toml file to manage your settings. The following parameters are required:

- key: Your API key
- secret: Your private key for the API
- amount: The amount to accumulate in JPY

Example config.toml:

```config.toml
key = "your_api_key"
secret = "your_secret_key"
amount = 1000 #JPY
```

## Usage

### Check Accumulation Feasibility

To check if accumulation is possible, run:

```bash
$ stashi check /path/to/config.toml
```

### Execute Accumulation

To perform the accumulation, run:

```bash
$ stashi run /path/to/config.toml
```

## Automating with Cron

To run stashi once a month, add the following line to your crontab:

```
0 6 1 * * stashi run /path/to/config.toml
```

This will execute the accumulation on the 1st of each month at 6:00 AM.