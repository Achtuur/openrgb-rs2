use openrgb2::{Color, DeviceType, OpenRgbClient, OpenRgbResult};

fn device_type_to_color(device_type: &DeviceType) -> openrgb2::Color {
    match device_type {
        DeviceType::Motherboard => Color::new(255, 0, 0), // red
        DeviceType::DRam => Color::new(0, 255, 0), // blue
        DeviceType::Gpu => Color::new(0, 0, 255), // green
        DeviceType::Cooler => Color::new(255, 255, 0), // yellow
        DeviceType::LEDStrip => Color::new(255, 0, 255), // magenta
        DeviceType::Keyboard => Color::new(0, 255, 255), // cyan
        DeviceType::Mouse => Color::new(255, 255, 255), // white
        DeviceType::MouseMat => Color::new(255, 128, 0), // orange
        DeviceType::Headset => Color::new(128, 0, 255), // purple
        DeviceType::HeadsetStand => Color::new(255, 0, 128), // pink
        DeviceType::Gamepad => Color::new(128, 255, 0), // light yellow
        DeviceType::Light => Color::new(0, 128, 255), // light blue
        DeviceType::Speaker => Color::new(0, 255, 128), // light green
        DeviceType::Virtual => Color::new(128, 128, 128), // gray
        DeviceType::Unknown => Color::new(0, 0, 0), // black
    }
}

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;

    let controllers = client.get_all_controllers().await?;

    let by_type = controllers.split_per_type();
    for (device_type, group) in by_type {
        let color = device_type_to_color(&device_type);
        println!("{:?}: {} controllers", device_type, group.len());
        for c in group {
            println!("  controller {}: {:#?}", c.id(), c.name());
            // set the LEDs to a specific color based on the device type
            c.set_leds(vec![color; c.num_leds()]).await?;
            c.set_all_leds(color).await?;
        }
    }

    Ok(())
}
