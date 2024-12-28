# ILayer Bot

ILayer Bot is a Rust application designed to fulfill ILayer orders efficiently.


## Installation

1. Navigate to the project directory:
    ```sh
    cd ilayer_bot
    ```
2. Install dependencies:
    ```sh
    cargo build
    ```    

## Usage

```sh
docker compose up -d # start the db
cp example.env .env # configure the RPC url and other configs
diesel setup # setup db and run migrations
cargo run # start the bot
```

## Migrations

```sh
diesel migration run
```
