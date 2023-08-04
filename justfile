
default: debug

debug:
    RUST_LOG=gitbounties_backend,info cargo run

devsetup:
    cp dev/hooks/* .git/hooks

fmt:
    cargo +nightly fmt --all

lint:
    cargo clippy -- -W clippy::unwrap_used -W clippy::cargo

dev_db:
    docker run --rm -p 8000:8000 surrealdb/surrealdb:latest start --log debug --ns test --db test --user admin --pass password memory
