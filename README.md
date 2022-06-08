# Firmware updates for embedded devices

[![CI](https://github.com/drogue-iot/embedded-update/actions/workflows/ci.yaml/badge.svg)](https://github.com/drogue-iot/embedded-update/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/v/embedded-update.svg)](https://crates.io/crates/embedded-update)
[![docs.rs](https://docs.rs/embedded-update/badge.svg)](https://docs.rs/embedded-update)
[![Matrix](https://img.shields.io/matrix/drogue-iot:matrix.org)](https://matrix.to/#/#drogue-iot:matrix.org)

The `embedded-update` crate implements a firmware update protocol for embedded devices connected to a firmware update service, which works in `no_std` (bare metal) environments.

Both the device to be updated and the update service are pluggable, so the protocol can be used with any device or service that implements the provided traits. This means you can use the library directly on an embedded device, or on a gateway that proxies multiple devices.

The library provides several update service reference implementations:

* [`Drogue Cloud`](https://github.com/drogue-iot/drogue-ajour) that works with the Drogue IoT open source project.
* [`Eclipse Hawkbit`](https://www.eclipse.org/hawkbit/) that works with the Eclipse Hawkbit DDI API.
* `InMemory` for testing.

Device side implementations can be found in [`Drogue Device`](https://github.com/drogue-iot/drogue-device), but these will gradually be migrated to `embedded-update` when types and traits for interacting with device flash is more proven. A `Simulator` device is provided for testing.

# Minimum supported Rust version (MSRV)

`embedded-update` requires two features from `nightly` to compile:

* `generic_associated_types`
* `type_alias_impl_trait`

These features are complete, but are not yet merged to `stable`.
