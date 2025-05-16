# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Note:** Version 0 of Semantic Versioning is handled differently from version 1 and above.
The minor version will be incremented upon a breaking change and the patch version will be incremented for features.

### Features:

## [0.2.4] - 2024-01-29

commit : a44923c33414f16fa0f378fb99c22b46de00ef74

- prio : Adding prioritization fees library [PR](https://github.com/blockworks-foundation/lite-rpc/pull/274)
- cluster-endpoints : GRPC optimizations [PR](multiples)
- quic : Fixing MTU bug [PR](https://github.com/blockworks-foundation/lite-rpc/pull/293)
- cluster-endpoints : Adding GRPC multiplexing [PR](https://github.com/blockworks-foundation/lite-rpc/pull/255)
- stake-vote : Calculating stake votes and leader schedule in lite-rpc [PR](https://github.com/blockworks-foundation/lite-rpc/pull/244) 

## [0.2.3] - 2023-09-23

commit : 3cdab51676a4b1bfb5b41739a383e30cd8a1c73c

- core : Refactored out solana rpc client from the core library. [PR](https://github.com/blockworks-foundation/lite-rpc/pull/174)
- services : Refactored out solana rpc client from the services library. [PR](https://github.com/blockworks-foundation/lite-rpc/pull/174)
- cluster-endpoints : Created cluster endpoint library and added rpc polling [PR](https://github.com/blockworks-foundation/lite-rpc/pull/174)
- cluster-endpoints : Added grpc support in cluster endpoints. [PR](https://github.com/blockworks-foundation/lite-rpc/pull/174)
- proxy : Added lite-rpc quic proxy which will act as a TPU forwarding proxy for multiple lite-rpc clients. [PR](https://github.com/blockworks-foundation/lite-rpc/pull/164) [PR](https://github.com/blockworks-foundation/lite-rpc/pull/169) [PR](https://github.com/blockworks-foundation/lite-rpc/pull/187)
- Using counting semaphore for block polling. [PR](https://github.com/blockworks-foundation/lite-rpc/pull/189)
- Changing algorithm for replay so that replay is done in linearly increasing time. [PR](https://github.com/blockworks-foundation/lite-rpc/pull/196)
- Unistream connection counting semaphore is now held by each connection. [PR](https://github.com/blockworks-foundation/lite-rpc/pull/194)
- Adding history libraries and block storing strategies. [PR](https://github.com/blockworks-foundation/lite-rpc/pull/205)
- Avoid sending transactions in TPUClient for which have expired blockheight. [PR](https://github.com/blockworks-foundation/lite-rpc/pull/204)

## [0.2.2] - 2023-06-23

commit : 70eb250b103c64a0e5a3159c9493e87003d046a4

- lite-rpc : Added restart logic.
- metrics : added more counters for related to failure of services during restart.
- tpu-client : sending transaction using multiple quic connections.
- tpu-client : removed pubsub of slot and implementing force polling using rpc.

## [0.2.1]

commit: c1eed987f29417f8a3b8d147f43a112388f02e4f

- postgres : removing postgres dependency on core and services
- all: refactored notification so that they do not fail, making the lite-rpc code and services work with other projects.

## [0.2.0]

commit: fffb302ce6f01ab0522a4ab23be60394bb9aa40f

- all: Seperating services that can be used by other projects into a library and creating a solana-lite-rpc-services package
- rpc: Creating a custom TPU Client instead of using solana TPU client more suitable for lite-rpc loads [PR](https://github.com/blockworks-foundation/lite-rpc/pull/105)
- metrics: Adding metrics related to custom tpu client [bf5841f](https://github.com/blockworks-foundation/lite-rpc/pull/105/commits/bf5841f43841d2bebd612abb714c53fbc920f090)

## [0.1.0] - 2023-04-01

commit: dc75e0e57386ce272bc22aa8fcfe35a0d8ce0eb0

Initial release.

### Includes

- rpc: Json rpc implementation using jsonrpsee crate to provide frontend to send and confirm transactions
- pubsub: A websocket implementation to subscribe to signature updates
- block-listening: A mechanism to get blocks from the RPC and read them to extract transaction data
- tpu-client: Mechanisms related to sending transaction to the cluster leaders
- postgres: Saving transaction and block data into postgres
- metrics: Updates related to metrics used for graphana and prometheus
- core: Core library,
- services: Services library
- lite-rpc: The lite rpc binary
- cluster-endpoints : Cluster endpoints library.
- proxy : Lite-rpc QUIC proxy to act as a forwarder
- prio : Prioritization fees libarary
- stake-vote : Stake vote library