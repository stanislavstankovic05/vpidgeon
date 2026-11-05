# Third-party licenses

vpidgeon itself is MIT licensed, see `LICENSE`. It uses the following crates, each under its own license. The license of every crate (and of the crates they use) is in its own package, and `cargo metadata` shows them all.

| Crate | Version | License |
|---|---|---|
| anyhow | 1.0.104 | MIT OR Apache-2.0 |
| chacha20poly1305 | 0.10.1 | Apache-2.0 OR MIT |
| crossterm | 0.29.0 | MIT |
| dotenvy | 0.15.7 | MIT |
| hkdf | 0.12.4 | MIT OR Apache-2.0 |
| ipconfig | 0.3.4 | MIT/Apache-2.0 |
| local-ip-address | 0.6.13 | MIT OR Apache-2.0 |
| num | 0.4.3 | MIT OR Apache-2.0 |
| rand | 0.10.3 | MIT OR Apache-2.0 |
| rand_core | 0.6.4 | MIT OR Apache-2.0 |
| ratatui | 0.30.2 | MIT |
| sha2 | 0.10.9 | MIT OR Apache-2.0 |
| snow | 0.10.0 | Apache-2.0 OR MIT |
| socket2 | 0.6.5 | MIT OR Apache-2.0 |
| ssh2 | 0.9.6 | MIT OR Apache-2.0 |
| terminal_size | 0.4.4 | MIT OR Apache-2.0 |
| whoami | 2.1.3 | Apache-2.0 OR BSL-1.0 OR MIT |
| x25519-dalek | 2.0.1 | BSD-3-Clause |
| zeroize | 1.9.0 | Apache-2.0 OR MIT |

The encryption of the messages is done by `snow`, an implementation of the Noise protocol (`Noise_NNpsk0_25519_ChaChaPoly_SHA256`).
