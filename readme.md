# TSL UMD protocols 🦀 📹
[docs](https://docs.rs/tsl-umd) | [crate](https://crates.io/crates/tsl-umd)

This is a Rust implementation of the TSL UMD family of tally protocols.

## Features
- [x] decoding
  - [x] v3.1
  - [ ] v4.0 (soon)
  - [ ] v5.0 (soon)
- [x] encoding - construct packets too
- [x] `no_std` - runs on anything
- [x] Zero copy (more or less) - fields are extracted from the buffer when you need them.
  If you don't need them, there's no overhead.

## Tested against
> [!NOTE]
> If you've successfully used this crate with some other hardware or software open an issue
> so we can add it here!

- [Tallyarbiter](https://josephdadams.github.io/TallyArbiter/)

## CLI
There's also a handy CLI utility! You need to specify the `cli` feature to actually get it:

`cargo install tsl-umd --features cli`

### Receiving TSL packets
The tool can listen on an address for incoming TSL packets and print them:
```
tslcli -t v3 listen --bind 0.0.0.0`
listening on 0.0.0.0:1234 for tsl V3 packets
got 18 bytes from 127.0.0.1:56989
[129, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
got packet addr=1, 1=false, 2=true, 3=false, 4=false, brightness=1, display=
got 18 bytes from 127.0.0.1:56989
[129, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
got packet addr=1, 1=false, 2=true, 3=false, 4=false, brightness=1, display=
got 18 bytes from 127.0.0.1:56989
[129, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
got packet addr=1, 1=false, 2=true, 3=false, 4=false, brightness=1, display=
```

### Sending
The tool can also send packets! E.g. to send a TSLv3 packet to `192.168.0.123` with the TSL
display address `13` and tally channels `1` and `2` on:
`tslcli -t v3 send --ip 192.168.0.123 --addr 13 --tally 1 --tally 2`
