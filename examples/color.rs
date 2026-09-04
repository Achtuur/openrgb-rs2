use std::time::Duration;

use openrgb2::{OpenRgbClient, OpenRgbResult};

/// R, G, B from 0.0 to 1.0 (common in shaders)
#[derive(Clone, Copy)]
pub struct Rgbaf32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Rgbaf32> for openrgb2::Color {
    fn from(value: Rgbaf32) -> Self {
        openrgb2::Color {
            r: (value.a * value.r * 255.0) as u8,
            g: (value.a * value.g * 255.0) as u8,
            b: (value.a * value.b * 255.0) as u8,
        }
    }
}

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;
    let controllers = client.get_all_controllers().await?;
    let controller = controllers.into_first()?;
    println!("Controller: {}", controller.name());

    controller.turn_off_leds().await?;
    // use custom color type, as long as it implements `Into<Color>`
    let my_color = Rgbaf32 {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    controller.set_led::<Rgbaf32>(0, my_color).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // works with iterators too
    let color_arr = [
        Rgbaf32 {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 0.1,
        },
        Rgbaf32 {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 0.4,
        },
        Rgbaf32 {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 0.9,
        },
    ];
    controller.set_leds(color_arr).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut cmd = controller.cmd();
    cmd.set_leds(color_arr)?;
    cmd.set_led(color_arr.len(), my_color)?;
    cmd.execute().await?;

    Ok(())
}
