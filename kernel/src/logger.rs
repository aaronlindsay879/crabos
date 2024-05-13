use log::{LevelFilter, Log, SetLoggerError};

use crate::serial_println;

pub static LOGGER: Logger = Logger::new(log::LevelFilter::Trace);

pub struct Logger {
    level: LevelFilter,
}

impl Logger {
    pub const fn new(level: LevelFilter) -> Self {
        Self { level }
    }

    pub fn init(&'static self) -> Result<(), SetLoggerError> {
        log::set_max_level(self.level);
        log::set_logger(self)
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level().to_level_filter() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            serial_println!("{:>5} | {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
