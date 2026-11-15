# Hydra

A bridge library between your trading server and crypto exchanges (Indodax, Binance, Tokocrypto, etc.), written in Rust.

## Overview

Hydra acts as a unified abstraction layer that lets your server communicate with multiple crypto exchanges through a single, consistent interface. Instead of dealing with each exchange's quirks and API differences directly, Hydra handles the translation.

```
Your Server  ──►  Hydra  ──►  Indodax
                       ├──►  Binance
                       ├──►  Tokocrypto
                       └──►  ... (more exchanges)
```

## Features

- Unified API across multiple exchanges
- Async-first, built on Tokio
- Precise decimal handling for trading amounts and prices
- Structured error handling and logging via [argus](https://github.com/saians07/argus.git)

## Project Structure

```
src/
├── apis/        # Exchange API clients
├── exchanges/   # Exchange-specific logic and abstractions
├── helpers/     # Shared utilities
└── lib.rs
```

## Requirements

- Rust (edition 2024)
- A `.env` file with your exchange API credentials (see `example.env`)

## Setup

1. Clone the repository:
   ```sh
   git clone https://github.com/saians07/hydra.git
   cd hydra
   ```

2. Copy the example env file and fill in your credentials:
   ```sh
   cp example.env .env
   ```

3. Build the project:
   ```sh
   cargo build
   ```

## Usage

Add Hydra as a dependency in your `Cargo.toml`:

```toml
[dependencies]
hydra = { git = "https://github.com/saians07/hydra.git" }
```

## Supported Exchanges

| Exchange   | Status      |
|------------|-------------|
| Indodax    | Planned     |
| Binance    | Planned     |
| Tokocrypto | Planned     |

## License

MIT
