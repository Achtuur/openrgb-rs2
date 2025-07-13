use openrgb2::{Color, OpenRgbClient, OpenRgbResult};

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;

    let controllers = client.get_all_controllers().await?;
    let controller = controllers
        .iter()
        .next()
        .expect("Must have at least one controller");

    println!("Performing example on {}", controller.name());
    let mut cmd = controller.cmd();
    // Set all LEDs to red
    cmd.set_leds(vec![Color::new(255, 0, 0); controller.num_leds()])?;
    // First zone to green
    cmd.set_zone_leds(
        0,
        vec![Color::new(0, 255, 0); controller.get_zone(0)?.num_leds()],
    )?;
    // First led to blue
    cmd.set_led(0, Color::new(0, 0, 255))?;
    // This is now equivalent to a single `controller.set_leds(...)` command
    cmd.execute().await?;

    Ok(())
}
