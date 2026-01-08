use core::fmt;

use clap::ValueEnum;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Environment {
    #[value(alias("dev"))]
    Development,
    #[value(alias("prod"))]
    Production,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}
