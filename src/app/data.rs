use std::fmt::Display;

use tui::{
    style::{Color, Style},
    widgets::ListItem,
};

use crate::sensor::SensorOutput;

#[derive(Debug, Default)]
pub struct SensorData<'a> {
    power_consumption: Vec<f64>,
    /// Total energy used so far, in kWh
    pub total_energy: f64,
    log: Vec<ListItem<'a>>,
}

impl SensorData<'_> {
    pub fn get_power_data(&self, points: usize) -> Box<[(f64, f64)]> {
        self.power_consumption
            .iter()
            .rev() // Reverse the iterator...
            .take(points) // so we can take the last 'points' samples
            .rev() // Reverse again
            .enumerate()
            .map(|(i, p)| (i as f64, *p))
            .collect()
    }

    pub fn get_log(&self) -> Vec<ListItem> {
        self.log.clone()
    }

    pub fn append_power_data(&mut self, power: f64) {
        self.power_consumption.push(power);
    }

    pub fn log_info<S>(&mut self, msg: S)
    where
        S: Display,
    {
        self.log
            .push(ListItem::new(format!("I: {}", msg)).style(Style::default().fg(Color::White)));
    }

    pub fn log_warn<S>(&mut self, msg: S)
    where
        S: Display,
    {
        self.log
            .push(ListItem::new(format!("W: {}", msg)).style(Style::default().fg(Color::LightRed)));
    }

    pub fn log_error<S>(&mut self, msg: S)
    where
        S: Display,
    {
        self.log
            .push(ListItem::new(format!("E: {}", msg)).style(Style::default().fg(Color::Red)));
    }

    pub fn handle_sensor_output(&mut self, out: &SensorOutput) {
        match out {
            SensorOutput::Log(msg) => self.log_info(msg),
            SensorOutput::Warn(msg) => self.log_warn(msg),
            SensorOutput::Error(msg) => self.log_error(msg),
            SensorOutput::Measurement(m) => {
                self.append_power_data(m.power);
                self.total_energy = m.energy
            }
        }
    }
}
