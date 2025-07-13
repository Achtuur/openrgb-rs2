openrgb-rs2 [![crates.io](https://img.shields.io/crates/v/openrgb.svg)](https://crates.io/crates/openrgb)
[![tests](https://github.com/Achtuur/openrgb-rs2/actions/workflows/tests.yml/badge.svg)](https://github.com/Achtuur/openrgb-rs2/actions/workflows/tests.yml)
==========

**Rust client library for the [OpenRGB SDK](https://gitlab.com/CalcProgrammer1/OpenRGB/-/blob/master/Documentation/OpenRGBSDK.md).**

[OpenRGB](https://openrgb.org/) is an RGB Lighting control app that doesn't depend on manufacturer software.

```rust
use openrgb2::{OpenRgbClient, OpenRgbResult};

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;

    let controllers = client.get_all_controllers().await?;
    for c in controllers {
        println!("controller {}: {:#?}", c.id(), c.name());
        // the LEDs should now be a rainbow
        c.init().await?;
    }

    Ok(())
}
```

See [documentation](https://docs.rs/openrgb2) and [examples](https://github.com/Achtuur/openrgb-rs2/tree/master/examples).

# Original `openrgb-rs`

This repository is a clone of the repo previously maintaed by [nicoulaj](https://github.com/nicoulaj/openrgb-rs). I have attempted to reach out to them, but received no response. As a result I decided to republish the OpenRGB SDK under a new name (`openrgb-rs2`).

## Whats different?

Support for OpenRGB protocol versions 4 and 5 is added. There's also now a friendlier to use API than before.

Internally there's some changes in how serializing/deserializing the protocol is done. I decided it was easier to read/write to a buffer, rather than directly to a stream as was previously done. For the end user there should not be much visible change though. I have not done any benchmarking, so I'm not sure about the performance. I can update my entire rig at about 300 FPS at release mode, so I'm not too worried about performance anyway.
