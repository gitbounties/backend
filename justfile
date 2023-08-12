
default: debug

debug:
    RUST_LOG=gitbounties_backend,info cargo run

devsetup:
    cp dev/hooks/* .git/hooks

fmt:
    cargo +nightly fmt --all

lint:
    cargo clippy -- -W clippy::unwrap_used -W clippy::cargo

db_up:
    docker run --rm -p 8000:8000 surrealdb/surrealdb:latest start --log debug --user admin --pass password memory

db_repl:
    surreal sql --conn http://localhost:8000 --user admin --pass password --ns test --db test

redis:
    docker run --rm -p 6379:6379 redis

certs:
    ./dev/scripts/certs
