use openrgb2::{Color, Led, OpenRgbClient, OpenRgbResult};

fn get_color_for_led(led: Led) -> Color {
    // Example logic to determine color based on LED ID
    let g = (led.id() as u8).wrapping_mul(10);
    Color::new(255, g, 0)
}

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;

    let controllers = client.get_all_controllers().await?;
    let controller = controllers
        .iter()
        .next()
        .expect("Must have at least one controller");

    println!("Selected Controller: {}", controller.name());

    for led in controller.led_iter() {
        println!("LED #{}: {} is {:?}", led.id(), led.name(), led.color());
    }

    // cmd api with iterator
    // you have to manually handle the error
    let cmd = controller
        .led_iter()
        .try_fold(controller.cmd(), |mut cmd, l| {
            cmd.set_led(l.id(), get_color_for_led(l))?;
            OpenRgbResult::Ok(cmd)
        })?;
    cmd.execute().await?;

    // previous cmd is equivalent to
    let cmd = controller.cmd_with_leds(get_color_for_led);
    cmd.execute().await?;
    Ok(())
}
