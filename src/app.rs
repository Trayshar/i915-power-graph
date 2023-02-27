use std::{
    io::{self, stdout},
    time::Duration, sync::mpsc::TryRecvError,
};

use crossterm::event::KeyEvent;
use tui::{backend::CrosstermBackend, Terminal};

use crate::{app::data::SensorData, sensor::SensorThread};

use self::inputs::{InputEvent, InputThread};

mod data;
mod inputs;
mod ui;

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

    let mut data = SensorData::default();
    loop {
        // Process sensor thread output
        loop {
            match sensor.read_next() {
                Ok(out) => data.handle_sensor_output(&out),
                // Do nothing, since the sensor thread didn't report anything new
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => data.log_error("Sensor thread died!"),
            }
        }

        // Render
        terminal.draw(|rect| ui::draw(rect, &data))?;

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

fn handle_key(_key: KeyEvent) -> AppReturn {
    AppReturn::Exit
}
