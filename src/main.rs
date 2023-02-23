use std::time::Duration;

mod app;
mod sensor;

fn main() -> Result<(), std::io::Error> {
    return app::start(Duration::from_millis(200), Duration::from_millis(500));

    let mut sensor = sensor::SensorThread::new(Duration::from_millis(500));
    loop {
        match sensor.read_next() {
            Ok(o) => println!("{:?}", o),
            Err(std::sync::mpsc::TryRecvError::Empty) => {},
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                println!("Sensor died!");
                return Ok(());
            }
        }
    }
}