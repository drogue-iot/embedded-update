# Firmware updates for embedded devices

[![CI](https://github.com/drogue-iot/embedded-update/actions/workflows/ci.yaml/badge.svg)](https://github.com/drogue-iot/embedded-update/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/v/embedded-update.svg)](https://crates.io/crates/embedded-update)
[![docs.rs](https://docs.rs/embedded-update/badge.svg)](https://docs.rs/embedded-update)
[![Matrix](https://img.shields.io/matrix/drogue-iot:matrix.org)](https://matrix.to/#/#drogue-iot:matrix.org)

The `embedded-update` crate implements a firmware update protocol for embedded devices connected to a firmware update service, which works in `no_std` (bare metal) environments.

Both the device to be updated and the update service are pluggable, so the protocol can be used with any device or service that implements the provided traits. This means you can use the library directly on an embedded device, or on a gateway that proxies multiple devices.

The library provides the `InMemory` and `Serial` reference implementations of the `UpdateService` trait, and the `Simulator` and `Serial` implementations for the `FirmwareDevice` trait.

Update service and device implementations can be added to `embedded-update` when types and traits for interacting with device flash and network connections are more widely available.

## Supported update services

* [`Drogue Cloud`](https://github.com/drogue-iot/drogue-ajour) by enabling the `drogue` feature.

## Supported devices

Device side implementations can be found in [`Drogue Device`](https://github.com/drogue-iot/drogue-device). 

# Minimum supported Rust version (MSRV)

`embedded-update` requires two features from `nightly` to compile:

* `generic_associated_types`
* `type_alias_impl_trait`

These features are complete, but are not yet merged to `stable`.
