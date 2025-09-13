use image::imageops::FilterType;
use inky_display::comm::{Inky, LedState};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut inky = Inky::new().unwrap();
    tracing::info!("Created Inky");
    inky.set_led(LedState::Off).unwrap();
    inky.set_led(LedState::On).unwrap();

    std::thread::sleep(Duration::from_millis(1000));

    let input_path = "./hello.png";
    let img = image::open(input_path).unwrap();
    let img = img.resize_exact(800, 480, FilterType::Nearest);
    let mut buf = img.to_rgb8();
    tracing::info!(
        "Using image '{}' with dimensions: {:?}",
        &input_path,
        buf.dimensions()
    );

    inky.set_display(&mut buf, false).unwrap();

    inky.set_led(LedState::Off).unwrap();
}
