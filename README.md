<div align="center">
  <img src="https://avatars.githubusercontent.com/u/134429862?s=200&v=4" width="150" />

  <h1>Libreplex</h1>

  <h4>
    <a href="https://libreplex.github.io/libreplex-program-library/">Documentation (stable)</a>
  </h4>
</div>



The mission of Libreplex is to provide a community-driven, open license protocols to the Solana SPL Token and NFT community.   The protocol must meet the following criteria:

1) Distributed deployment keys

To ensure that no single entity can unilaterally make changes that impact or jeopardise the integrity of the applications that depend on the protocol.

2) Open license held by a Trust / Foundation

The licensing must ensure that any applications utilising the protocol can do so knowing that the nature of the protocol remains constant, to minimise uncertainty and maximise transparency.

3) Guaranteed fees-free for life

The fee-free nature of the protocol ensures that even though applications built on top of the protocol may introduce fees, the protocol itself will never do so.  This establishes a level playing field to all and enforces predictability and transparency.

 4) Open source

The source of the protocol will be made available on github or similar.z

INSTRUCTIONS:

Install dependencies

```
yarn
```

Build

```
anchor build
```

To run unit tests (cargo):

`cargo test`

To run unit tests (cargo, for a single program):

`cargo test libreplex_metadata`