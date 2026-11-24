# Changelog

What each release changed for you, newest first. Each line is a commit's summary, linked to its full description and diff.

## 2.0.8 - 2026-11-24

### Features

- Require internetdata 3.4.0: a condition on a list member matches any element ([`4ed46e2`](https://github.com/internetdata/sdk-rust-axum/commit/4ed46e259c42bdcd60c19c0e2dea0def085a3b3b))

## 2.0.7 - 2026-11-23

### Fixes

- Require futures-util 0.3.31, past RUSTSEC-2020-0062 ([`3821baf`](https://github.com/internetdata/sdk-rust-axum/commit/3821bafec2695640b56803f7fb3a556b7479a01d))
- Require internetdata 3.3.2: futures-util 0.3.31, past RUSTSEC-2020-0062 ([`0f684c0`](https://github.com/internetdata/sdk-rust-axum/commit/0f684c0a0482d5a726f8f65f415006e7db6ea6a2))

## 2.0.6 - 2026-11-23

### Fixes

- Require internetdata 3.3.1, on spec 2026.11.22 ([`2409816`](https://github.com/internetdata/sdk-rust-axum/commit/240981660061be9ce216903f9a7d7b8608d40790))

## 2.0.5 - 2026-11-22

### Features

- Require internetdata 3.3.0: is_bot on every answer, and a paid answer's bot ([`98aaecd`](https://github.com/internetdata/sdk-rust-axum/commit/98aaecd110ac119e65dd5b9da91488def807f7f9))

## 2.0.4 - 2026-11-20

### Fixes

- Require internetdata 3.2.0: a paid domain answer's hosting, co-hosted domains, home page, redirects and backlinks ([`867183a`](https://github.com/internetdata/sdk-rust-axum/commit/867183aceebe3bf225984b73f3ede9897239d920))

## 2.0.3 - 2026-11-20

### Fixes

- Require internetdata 3.1.2: a reduced answer is cached a minute at most ([`bb2f90e`](https://github.com/internetdata/sdk-rust-axum/commit/bb2f90e611076f899aa7222ef5057417fa681f57))

## 2.0.2 - 2026-11-18

### Fixes

- Require internetdata 3.1.1: the spec as republished ([`285671f`](https://github.com/internetdata/sdk-rust-axum/commit/285671f5c717889e0b405e93be07f8e1913f4557))

## 2.0.1 - 2026-11-17

### Features

- Require internetdata 3.1.0: cap_reached on a paid answer past its overage cap ([`5f010c0`](https://github.com/internetdata/sdk-rust-axum/commit/5f010c0488add256cd5f74fbd3abcba9c60af297))

## 2.0.0 - 2026-11-16

### Breaking changes

- Require internetdata 3.0.0: a vocabulary value it was not built with comes back as served ([`3026fb9`](https://github.com/internetdata/sdk-rust-axum/commit/3026fb99f18138d937cb95c33a8ec33f69f940e1))

## 1.0.3 - 2026-11-16

### Features

- Require internetdata 2.12.0: an AS's rank, neighbors and hosted domains ([`5b577b8`](https://github.com/internetdata/sdk-rust-axum/commit/5b577b8292fecb40c70e68b327bcd572694dead4))

## 1.0.2 - 2026-11-15

### Features

- Require internetdata 2.11.0: hosted_domains lists its 10 most popular domains ([`c3f02a7`](https://github.com/internetdata/sdk-rust-axum/commit/c3f02a7e7c223a196791bf477acf4f60ef986384))

## 1.0.1 - 2026-11-14

### Fixes

- Require internetdata 2.10.1: a 403 can mean the key lacks the ip.lookup scope ([`3cd8d6c`](https://github.com/internetdata/sdk-rust-axum/commit/3cd8d6ca7b85c8215f58f8671134e3b1db6cc48f))

## 1.0.0 - 2026-11-14

### Features

- Attach the visitor's InternetData record to every request, read with the `Found` extractor: the location, the network and the company behind the address, and the anonymity and hosting flags your plan serves ([`03d1649`](https://github.com/internetdata/sdk-rust-axum/commit/03d1649240473212a5280a8bceab9d3f99442f3f))
- Refuse a request with a `block_condition` written in the shape of a result or read from JSON, by country, network type, anonymity or any other member, with 403 or an `on_blocked` of your own ([`03d1649`](https://github.com/internetdata/sdk-rust-axum/commit/03d1649240473212a5280a8bceab9d3f99442f3f))
- Choose where the client address comes from: axum's `ConnectInfo` by default, `xff`, `header` for a header your edge writes, or a closure of your own ([`03d1649`](https://github.com/internetdata/sdk-rust-axum/commit/03d1649240473212a5280a8bceab9d3f99442f3f))
- Fail open on any lookup failure unless `fail_closed` is set, with a 2500 ms bound, no retry and a cache per layer by default ([`03d1649`](https://github.com/internetdata/sdk-rust-axum/commit/03d1649240473212a5280a8bceab9d3f99442f3f))
