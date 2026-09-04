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

    let biggest = controllers
        .iter()
        .max_by(|c, c2| c.num_leds().cmp(&c2.num_leds()))
        .expect("Must have at least one controller");

    // goal is to set every led at once
    // individual set led
    let timer = std::time::Instant::now();
    for i in 0..2 {
        println!("Individual set leds #{i}");
        for led in 0..biggest.num_leds() {
            let color = RAINBOW_COLORS[(i + led) % RAINBOW_COLORS.len()];
            biggest.set_led(led, color).await?;
        }
    }
    println!("Individual set: {:?}", timer.elapsed());

    println!("Commands");
    // command group
    let timer = std::time::Instant::now();
    for i in 0..2 {
        println!("Commands run #{i}");
        let mut cmd = biggest.cmd();
        for led in 0..biggest.num_leds() {
            let color = RAINBOW_COLORS[(i + led) % RAINBOW_COLORS.len()];
            cmd.set_led(led, color)?;
        }
        cmd.execute().await?;
    }
    println!("Commands: {:?}", timer.elapsed());

    Ok(())
}
