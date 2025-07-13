use openrgb2::{Color, OpenRgbClient, OpenRgbResult};

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;
    let group = client.get_all_controllers().await?;
    let mut cmd = group.cmd();
    for c in &group {
        cmd.set_controller_leds(c, vec![Color::new(255, 0, 0); c.num_leds()])?;
    }
    cmd.execute().await?;
    Ok(())
}
