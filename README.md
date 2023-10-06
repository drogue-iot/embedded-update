# Firmware updates for embedded devices

[![CI](https://github.com/drogue-iot/embedded-update/actions/workflows/ci.yaml/badge.svg)](https://github.com/drogue-iot/embedded-update/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/v/embedded-update.svg)](https://crates.io/crates/embedded-update)
[![docs.rs](https://docs.rs/embedded-update/badge.svg)](https://docs.rs/embedded-update)
[![Matrix](https://img.shields.io/matrix/drogue-iot:matrix.org)](https://matrix.to/#/#drogue-iot:matrix.org)

The `embedded-update` crate implements firmware update protocol for embedded devices connected to a firmware update service, which works in `no_std` environments.

Both the device to be updated and the update service are pluggable, so the protocol can be used with any device flash or service that implements the provided traits. This means you can use the library directly on an embedded device, or on a gateway that proxies requests across multiple devices.

The library provides the `InMemory` and `Serial` reference implementations of the `UpdateService` trait, and the `Simulator` and `Serial` implementations for the `FirmwareDevice` trait.

Update service and device implementations can be added to `embedded-update` when types and traits for interacting with device flash and network connections are more widely available.

## Supported update services

* (builtin) `Serial` - implements a serial update protocol for a device, that can be used over UART, USB Serial etc.
* (builtin) `InMemory` - implements a hard coded update service that serves an update from memory.
* (external) [Drogue Device `HttpUpdater`](https://github.com/drogue-iot/drogue-device) - implements an update protocol over HTTP using Drogue Cloud + Drogue Ajour.
* (external) [Drogue Device `LorawanService`](https://github.com/drogue-iot/drogue-device) - implements an update protocol over LoRaWAN using Drogue Cloud + Drogue Ajour.

## Supported devices

* (builtin) `Serial` - implements a serial update protocol allowing to talk to a device implementing this protocol over UART, USB Serial etc.
* (builtin) `Simulated` - implements a simulated device for testing update services.

# Minimum supported Rust version (MSRV)

`embedded-update` requires a feature from `nightly` to compile when using the `nightly` flag.

* async_fn_in_traits
