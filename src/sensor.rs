use std::{
    ffi::{OsStr, OsString},
    fs,
    io::{Read, Seek},
    sync::mpsc::{channel, Receiver, TryRecvError},
    thread,
    time::Duration,
};

#[derive(Debug)]
pub struct MeasurementPoint {
    /// Accumulated energy consumption (kWh)
    pub energy: f64,
    /// Current power draw (Watt)
    pub power: f64,
}

#[derive(Debug)]
pub enum SensorOutput {
    Log(String),
    Warn(String),
    Error(String),
    Measurement(MeasurementPoint),
}

// Small macro to reduce logging boilerplate
macro_rules! tx_log {
    // Single line logging
    ($tx:expr, $level:ident, $msg:expr) => {
        if $tx.send(
            SensorOutput::$level($msg.into())
        ).is_err() {
            // Stops this thread if we cannot write to our main thread (it likely died already)
            return;
        }
    };
    // Logging with format macro
    ($tx:expr, $level:ident, $msg:expr, $( $arg:expr ),*) => {
        if $tx.send(
            SensorOutput::$level(
                format!($msg, $($arg,)*)
            )
        ).is_err() {
            // Stops this thread if we cannot write to our main thread (it likely died already)
            return;
        }
    }
}

fn append<I, T>(path: I, append: T) -> OsString
where
    I: Into<OsString>,
    T: AsRef<OsStr>,
{
    let mut p: OsString = path.into();
    p.push(append);
    p
}

pub struct SensorThread {
    results: Receiver<SensorOutput>,
}

impl SensorThread {
    pub fn new(tick_rate: Duration) -> SensorThread {
        let (tx, rx) = channel();

        thread::spawn(move || loop {
            let hwmon_i915_dir = {
                tx_log!(tx, Log, "Looking for i915 sensor hwmon directory");
                match fs::read_dir("/sys/class/hwmon") {
                    Ok(mut dirs) => {
                        loop {
                            // Look for '/sys/class/hwmon/hwmon{?}/name' file with 'i915' in it
                            match dirs.next() {
                                // Found valid subfolder
                                Some(Ok(hwmon_sensor_dir)) => {
                                    match fs::read_to_string(append(
                                        hwmon_sensor_dir.path(),
                                        "/name",
                                    )) {
                                        Ok(name) => {
                                            if name.strip_suffix("\n").unwrap_or(&name) == "i915" {
                                                tx_log!(
                                                    tx,
                                                    Log,
                                                    "Found i915 energy sensor at {:?}",
                                                    hwmon_sensor_dir.path()
                                                );
                                                break hwmon_sensor_dir.path();
                                            }
                                        }
                                        Err(e) => tx_log!(
                                            tx,
                                            Warn,
                                            "Failed to read 'name' file in hwmon subdirectory: {}",
                                            e
                                        ),
                                    }
                                }
                                // Invalid subfolder
                                Some(Err(e)) => {
                                    tx_log!(tx, Warn, "Failed to access hwmon subdirectory: {}", e)
                                }
                                // Iterator is empty, and we are still in the loop
                                None => {
                                    tx_log!(tx, Error, "Failed to find i915 sensor!");
                                    // Exit this thread, we cannot operate like this
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tx_log!(tx, Error, "Failed to open \"/sys/class/hwmon\": {}", e);
                        // Exit this thread, we cannot operate like this
                        return;
                    }
                }
            };
            let mut hwmon_i915_energy_file =
                match fs::File::open(append(hwmon_i915_dir, "/energy1_input")) {
                    Ok(f) => f,
                    Err(e) => {
                        tx_log!(tx, Error, "Failed to open i915 energy sensor file: {}", e);
                        // Exit this thread, we cannot operate like this
                        return;
                    }
                };
            // Define this function as a macro so I can kill the thread
            let mut buf = String::new();
            macro_rules! read_energy_file {
                () => { // No macro arguments
                    {
                        // Return the read position of this file handle to 0, so we can read the whole file again
                        match hwmon_i915_energy_file.seek(std::io::SeekFrom::Start(0)) {
                            Ok(0) => {},
                            _ => {
                                tx_log!(tx, Error, "Failed to seek to 0 in i915 energy sensor file");
                                // Exit this thread, we cannot operate like this
                                return;
                            }
                        }
                        // Clear buffer and read file
                        buf.clear();
                        match hwmon_i915_energy_file.read_to_string(&mut buf) {
                            Ok(_) => {},
                            Err(e) => {
                                tx_log!(tx, Error, "Failed to read i915 energy sensor file: {}", e);
                                // Exit this thread, we cannot operate like this
                                return;
                            }
                        }
                        // Parse string representation into number
                        match buf.strip_suffix("\n").unwrap_or(&buf).parse::<u64>() {
                            // Return energy in joules
                            Ok(n) => n / 1_000_000, // microjoule -> J
                            Err(e) => {
                                tx_log!(tx, Error, "Failed to parse i915 energy sensor file (content: \"{}\"): {}", buf, e);
                                // Exit this thread, we cannot operate like this
                                return;
                            }
                        }
                    }
                }
            }

            // Declare accumulated energy reading for later reference in Joule
            let mut last_energy = read_energy_file!();
            thread::sleep(tick_rate);

            loop {
                let energy: u64 = read_energy_file!(); // Joule
                let power = (energy - last_energy) as f64 / tick_rate.as_secs_f64(); // Watt
                last_energy = energy;

                tx.send(SensorOutput::Measurement(MeasurementPoint {
                    energy: energy as f64 / (3.6 * 1_000_000.0), // J -> kWh
                    power
                }))
                .unwrap();
                thread::sleep(tick_rate);
            }
        });

        SensorThread { results: rx }
    }

    pub fn read_next(&self) -> Result<SensorOutput, TryRecvError> {
        self.results.try_recv()
    }
}
