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
cargo run # start the bot
```

## Testing

To run unit tests, use the following command:
```sh
cargo test
```

To run end-to-end (e2e) tests, use the following command:
```sh
cargo test -- --ignored
```
Note that e2e tests have a dependency on `docker-compose` and `anvil` to start the necessary services.

## Migrations

```sh
sea-orm-cli migrate refresh # NB. refresh undoes-redoes every migration, only for dev env
```
