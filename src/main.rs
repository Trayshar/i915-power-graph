use std::time::Duration;

mod app;
mod sensor;

fn main() -> Result<(), std::io::Error> {
    app::start(Duration::from_millis(250), Duration::from_millis(500))
}
