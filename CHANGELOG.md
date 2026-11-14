# Changelog

What each release changed for you, newest first. Each line is a commit's summary, linked to its full description and diff.

## 1.0.0 - 2026-11-14

### Features

- Attach the visitor's InternetData record to every request, read with the `Found` extractor: the location, the network and the company behind the address, and the anonymity and hosting flags your plan serves ([`03d1649`](https://github.com/internetdata/sdk-rust-axum/commit/03d1649240473212a5280a8bceab9d3f99442f3f))
- Refuse a request with a `block_condition` written in the shape of a result or read from JSON, by country, network type, anonymity or any other member, with 403 or an `on_blocked` of your own ([`03d1649`](https://github.com/internetdata/sdk-rust-axum/commit/03d1649240473212a5280a8bceab9d3f99442f3f))
- Choose where the client address comes from: axum's `ConnectInfo` by default, `xff`, `header` for a header your edge writes, or a closure of your own ([`03d1649`](https://github.com/internetdata/sdk-rust-axum/commit/03d1649240473212a5280a8bceab9d3f99442f3f))
- Fail open on any lookup failure unless `fail_closed` is set, with a 2500 ms bound, no retry and a cache per layer by default ([`03d1649`](https://github.com/internetdata/sdk-rust-axum/commit/03d1649240473212a5280a8bceab9d3f99442f3f))
