# 42 Get Leaderboard

Small Rust CLI that builds a leaderboard for a 42 cohort from the Intra API.

It is currently tailored to one use case: campus `1`, pool year `2022`, and the July/August/September pool months.

## How It Works

1. Fetches the first 5 pages of `/v2/campus/1/users`.
2. Keeps only active users.
3. Fetches each active profile individually.
4. Extracts the `42cursus` level from `cursus_users`.
5. Sorts the result descending and prints a text leaderboard.

The CLI also rate-limits itself to roughly 2 requests per second while fetching profile details.

## Run

```bash
export TOKEN="your-42-intra-bearer-token"
cargo run
```

## Build

```bash
cargo build --release
```

## Limitations

- No CLI flags yet: campus, pool, and page count are hardcoded in `src/main.rs`.
- API/auth failures currently abort the program instead of being handled gracefully.
- The output is a terminal ranking, not a CSV or web view.
