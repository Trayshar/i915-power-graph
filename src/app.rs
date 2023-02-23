use std::{
    io::{self, stdout},
    time::Duration,
};

use crossterm::event::KeyEvent;
use tui::{backend::CrosstermBackend, Terminal};

use crate::sensor::SensorThread;

use self::inputs::{InputEvent, InputThread};

mod inputs;
mod ui;

pub const GRAPH_DATA_LEN: usize = 100;
pub type PowerGraph = [(f64, f64); GRAPH_DATA_LEN];

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub fn start(ui_tick_rate: Duration, sensor_tick_rate: Duration) -> Result<(), io::Error> {
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let inputs = InputThread::new(ui_tick_rate);
    let sensor = SensorThread::new(sensor_tick_rate);

    let mut log: Vec<String> = vec![];

    let mut power_data: PowerGraph = [(0.0, 0.0); GRAPH_DATA_LEN];
    let mut index: usize = 0;

    const J_TO_KWH: f64 = 1.0 / (3.6 * 1_000_000.0);
    let mut total_energy: f64 = 0.0;
    loop {
        // Process sensor thread
        loop {
            match sensor.read_next() {
                Ok(crate::sensor::SensorOutput::Measurement(p)) => {
                    power_data[index] = (index as f64, p.power);
                    index = (index + 1) % GRAPH_DATA_LEN;
                    power_data[index] = (index as f64, 0.0); // Clear next value

                    total_energy = p.energy as f64 * J_TO_KWH;
                }
                // TODO: Improve logging
                Ok(o) => {log.push(format!("{:?}", o))},
                // Do nothing, since the sensor thread didn't report anything new
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                // Sensor thread died
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    log.push("Sensor died!".to_owned())
                }
            }
        }

        // Render
        terminal.draw(|rect| ui::draw(rect, &log, &power_data))?;

        // Handle inputs
        let result = match inputs.next() {
            Err(_) => AppReturn::Exit, // Input handler died?!
            Ok(InputEvent::Key(key)) => handle_key(key),
            Ok(InputEvent::None) => AppReturn::Continue,
        };

        // Check if we should exit
        if result == AppReturn::Exit {
            break;
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn handle_key(key: KeyEvent) -> AppReturn {
    AppReturn::Exit
}