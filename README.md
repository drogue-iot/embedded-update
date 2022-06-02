# Firmware updater for embedded devices

[![CI](https://github.com/drogue-iot/firmware-updater/actions/workflows/ci.yaml/badge.svg)](https://github.com/drogue-iot/firmware-updater/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/v/firmware-updater.svg)](https://crates.io/crates/firmware-updater)
[![docs.rs](https://docs.rs/firmware-updater/badge.svg)](https://docs.rs/firmware-updater)
[![Matrix](https://img.shields.io/matrix/drogue-iot:matrix.org)](https://matrix.to/#/#drogue-iot:matrix.org)

The `firmware-updater` crate implements a firmware update protocol for embedded devices connected to a firmware update service, which works in `no_std` (bare metal) environments.

Both the device to be updated and the update service are pluggable, so the protocol can be used with any device or service that implements the provided traits. This means you can use the library directly on an embedded device, or on a gateway that proxies multiple devices.

The library provides a reference update service implementation for [`Drogue Cloud`](https://github.com/drogue-iot/drogue-ajour), as well as an `InMemory` type for testing. For device side implementations, these can be found in [`Drogue Device`](https://github.com/drogue-iot/drogue-device), but a `Simulator` device is provided for testing.

# Minimum supported Rust version (MSRV)

`firmware-updater` requires two features from `nightly` to compile:

* `generic_associated_types`
* `type_alias_impl_trait`

These features are complete, but are not yet merged to `stable`.
