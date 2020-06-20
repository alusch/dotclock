# dotclock

Command-line tool to show a clock on a Luminator display, written in Rust.

At the moment, it's really only *my* Luminator display (a MAX3000 90 × 7 side sign at address 3), but it would be relatively easy to extend to different types of signs. If nothing else, it's a good example of how to use my [flipdot](https://github.com/alusch/flipdot) crate to do something useful.

## Usage

The most common "real-world" usage is

```
./dotclock /dev/ttyUSB0
```

which will attempt communication with an actual sign over the specified serial port. You can use the `-t` or `--24hour` switches to display a 24-hour representation instead of the default 12-hour.

For testing purposes, you can pass `virtual` as the port name to fake communication with a virtual sign instead. This doesn't actually print anything to the console without enabling the `RUST_LOG` environment variable. Example:

```
RUST_LOG=flipdot=info ./dotclock virtual
```

## License

Distributed under the [MIT license](/LICENSE).

Note: Depends on the [`timer`](https://github.com/Yoric/timer.rs) crate, which is licensed under MPL 2.0.
