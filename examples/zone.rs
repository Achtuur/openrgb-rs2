use openrgb2::{Color, DeviceType, OpenRgbClient, OpenRgbResult};

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;
    let group = client
        .get_controllers_of_type(DeviceType::Motherboard)
        .await?;
    let controller = group.into_first().expect("No motherboard controller found");
    println!("Controller: {}", controller.name());

    let zone = controller.get_zone(0)?;
    println!("Zone: {}", zone.name());

    zone.set_all_leds(Color::new(255, 0, 0)).await?;
    Ok(())
}
