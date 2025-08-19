use openrgb2::{OpenRgbClient, OpenRgbResult};

#[tokio::main]
async fn main() -> OpenRgbResult<()> {
    // connect to local server
    let client = OpenRgbClient::connect().await?;

    let controllers = client.get_all_controllers().await?;
    for c in controllers {
        println!("controller {}: {:#?}", c.id(), c.name());
        // the LEDs should now be a rainbow
        c.init().await?;
    }

    Ok(())
}

