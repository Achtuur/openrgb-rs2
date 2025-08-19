openrgb-rs2 [![crates.io](https://img.shields.io/crates/v/openrgb.svg)](https://crates.io/crates/openrgb)
[![tests](https://github.com/Achtuur/openrgb-rs2/actions/workflows/tests.yml/badge.svg)](https://github.com/Achtuur/openrgb-rs2/actions/workflows/tests.yml)
==========

**Rust client library for the [OpenRGB SDK](https://gitlab.com/CalcProgrammer1/OpenRGB/-/blob/master/Documentation/OpenRGBSDK.md).**

[OpenRGB](https://openrgb.org/) is an RGB Lighting control app that doesn't depend on manufacturer software.

See [documentation](https://docs.rs/openrgb2) and [examples](https://github.com/Achtuur/openrgb-rs2/tree/master/examples).

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

# Performance

The OpenRGB SDK provides a few ways to write colors to devices: [per led](https://gitlab.com/CalcProgrammer1/OpenRGB/-/blob/master/Documentation/OpenRGBSDK.md#net_packet_id_rgbcontroller_updateleds), [per zone](https://gitlab.com/CalcProgrammer1/OpenRGB/-/blob/master/Documentation/OpenRGBSDK.md#net_packet_id_rgbcontroller_updatezoneleds), or [all leds (`set_leds`)](https://gitlab.com/CalcProgrammer1/OpenRGB/-/blob/master/Documentation/OpenRGBSDK.md#net_packet_id_rgbcontroller_updateleds). From my testing it seemed that when updating large number of LEDs, `set_leds` is the fastest one. It's inconvenient to update in this way, as some controllers have multiple unrelated zones, such as motherboards, meaninig you have to keep track of the zone offset.


## Command API

To ease this, the crate has a `Command` API, which translates arbitrary LED updates to a single `all_leds` update, ensuring both user friendliness and maximum performance. The entire API is sync, with only one asynchronous api call at the end.


```rust
use openrgb2::{OpenRgbClient, OpenRgbResult, Color};

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;

    // get a controller
    let controllers = client.get_all_controllers().await?;
    let controller = controllers
        .iter()
        .next()
        .expect("Must have at least one controller");
    controller.init().await?;

    let mut cmd = controller.cmd();
    // Set all LEDs to red
    cmd.set_leds(vec![Color::new(255, 0, 0); controller.num_leds()])?;
    // First half of first zone to green
    cmd.set_zone_leds(
        0,
        vec![Color::new(0, 255, 0); controller.get_zone(0)?.num_leds() / 2],
    )?;
    // First led to blue
    cmd.set_led(0, Color::new(0, 0, 255))?;
    // This is now equivalent to a single `controller.set_leds(...)` command
    cmd.execute().await?;
    Ok(())
}
```

### Multiple devices

My case contains a few controllers that I would like to control in sync, like my RAM sticks and case fans. To make it easier to update those, the `Command` API also supports `ControllerGroup`s.

```rust
use openrgb2::{Color, OpenRgbClient, OpenRgbResult};

const RAINBOW_COLORS: [Color; 7] = [
    Color::new(255, 0, 0),   // Red
    Color::new(255, 127, 0), // Orange
    Color::new(255, 255, 0), // Yellow
    Color::new(0, 255, 0),   // Green
    Color::new(0, 0, 255),   // Blue
    Color::new(85, 0, 180),  // Indigo
    Color::new(148, 0, 211), // Violet
];

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;
    let group = client.get_all_controllers().await?;
    group.init().await?;
    // sets each separate device to a color of the rainbow
    let mut cmd_group = group.cmd();
    for (idx, c) in group.iter().enumerate() {
        let color = RAINBOW_COLORS[idx % RAINBOW_COLORS.len()];
        cmd_group.set_controller_leds(c, vec![color; c.num_leds()])?;
    }
    // executes a `set_leds` for every controller
    cmd_group.execute().await?;
    Ok(())
}
```


# Original `openrgb-rs`

This repository is a clone of the repo previously maintaed by [nicoulaj](https://github.com/nicoulaj/openrgb-rs). I have attempted to reach out to them, but received no response. As a result I decided to republish the OpenRGB SDK under a new name (`openrgb-rs2`).

## Whats different?

Support for OpenRGB protocol versions 4 and 5 is added. There's also now a friendlier to use API than before.

Internally there's some changes in how serializing/deserializing the protocol is done. I decided it was easier to read/write to a buffer, rather than directly to a stream as was previously done. For the end user there should not be much visible change though. I have not done any benchmarking, so I'm not sure about the performance. I can update my entire rig at about 300 FPS at release mode, so I'm not too worried about performance anyway.
