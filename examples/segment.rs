use openrgb2::{Color, DeviceType, OpenRgbClient, OpenRgbResult};

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;
    let group = client.get_controllers_of_type(DeviceType::Motherboard).await?;
    let controller = group.into_first().expect("No motherboard controller found");
    println!("Controller: {}", controller.name());

    let zone = controller.get_zone(1)?;
    println!("Zone: {}", zone.name());

    // You can add segments to (some) zones in OpenRGB.
    let segment = zone.get_segment(0)?;
    println!("Segment: {}", segment.name());

    // this will set the rest of the zone LEDs to black
    segment.set_all_leds(Color::new(0, 255, 0)).await?;

    // instead we can do:
    let mut cmd = segment.cmd();
    // set "background" by specifying colors for the zone
    cmd.set_zone_leds(
        zone.id(),
        vec![Color::new(255, 0, 0); zone.num_leds()]
    )?;
    // set segment
    cmd.set_segment_leds(
        zone.id(), segment.segment_id(),
        vec![Color::new(0, 255, 0); segment.num_leds()] // green segment
    )?;
    cmd.execute().await?;
    Ok(())
}
