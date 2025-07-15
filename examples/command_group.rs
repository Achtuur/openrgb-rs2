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
    let mut cmd = group.cmd();
    for (idx, c) in group.iter().enumerate() {
        let color = RAINBOW_COLORS[idx % RAINBOW_COLORS.len()];
        cmd.set_controller_leds(c, vec![color; c.num_leds()])?;
    }
    cmd.execute().await?;
    Ok(())
}
