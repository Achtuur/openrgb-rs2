use openrgb2::{Color, OpenRgbClient, OpenRgbResult};

const RAINBOW_COLORS: [Color; 7] = [
    Color::new(255, 0, 0),   // Red
    Color::new(255, 127, 0), // Orange
    Color::new(255, 255, 0), // Yellow
    Color::new(0, 255, 0),   // Green
    Color::new(0, 0, 255),   // Blue
    Color::new(75, 0, 130),  // Indigo
    Color::new(148, 0, 211), // Violet
];

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;
    let controllers = client.get_all_controllers().await?;
    let mut colors = RAINBOW_COLORS.iter().cycle();

    let biggest = controllers
        .iter()
        .max_by(|c, c2| c.num_leds().cmp(&c2.num_leds()))
        .expect("Must have at least one controller");

    // goal is to set every led at once
    // individual set led
    let timer = std::time::Instant::now();
    for i in 0..10 {
        for led in 0..biggest.num_leds() {
            let color = colors.nth(i + led).unwrap();
            biggest.set_led(led, *color).await?;
        }
    }
    println!("Individual set: {:?}", timer.elapsed());

    println!("Commands");
    // command group
    let timer = std::time::Instant::now();
    for i in 0..10 {
        let mut cmd = biggest.cmd();
        for led in 0..biggest.num_leds() {
            let color = colors.nth(i + led).unwrap();
            cmd.set_led(led, *color)?;
        }
        cmd.execute().await?;
    }
    println!("Commands: {:?}", timer.elapsed());

    let timer = std::time::Instant::now();
    for i in 0..10 {
        let timer = std::time::Instant::now();
        let mut cmd_group = controllers.cmd();
        for controller in controllers.iter() {
            for zone in controller.get_all_zones() {
                for led in 0..zone.num_leds() {
                    let color = colors.nth(i + led).unwrap();
                    cmd_group.set_controller_zone_led(controller, zone.id(), led, *color)?;
                }
            }
        }
        cmd_group.execute().await?;
        println!("{:?}", timer.elapsed());
    }
    println!("Commands for all devices: {:?}", timer.elapsed());

    Ok(())
}
