
use std::time::SystemTime;

#[macro_use]
extern crate tracing;
use tracing::{span, Level};

extern crate structopt;
use structopt::StructOpt;

extern crate serde;
use serde::{Serialize, Deserialize};

extern crate toml;

extern crate humantime;
use humantime::Duration;

extern crate streamdeck;
use streamdeck::{StreamDeck, Error as DeckError};

pub mod automata;
pub use automata::*;

/// Automation configuration options
#[derive(StructOpt)]
pub struct Options {
    #[structopt(long, default_value="100ms", env="BLOCK_PERIOD")]
    /// Period for blocking polling of input device
    block_period: Duration,

    #[structopt(long, default_value="5m", env="POLL_PERIOD")]
    /// Period for running automata update functions
    poll_period: Duration,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Toml(toml::de::Error),
    Deck(streamdeck::Error),
    InvalidState,
    TooManyAutomata,
}

/// Automation object contains a collection of automata
#[derive(Debug, Serialize, Deserialize)]
pub struct Automation {
    automata: Vec<Automata>,
}

impl Automation {
    /// Load an automation from a file
    pub fn load(file: &str) -> Result<Self, Error> {
        let d = std::fs::read_to_string(file).map_err(Error::Io)?;
        let a = toml::from_str(&d).map_err(Error::Toml)?;

        Ok(a)
    }

    /// Execute an automation
    pub fn run(&mut self, deck: &mut StreamDeck, opts: Options) -> Result<(), Error> {
        debug!("Executing automation");

        if self.automata.len() > deck.kind().keys() as usize {
            error!("Specified number of automata exceed available keys");
            return Err(Error::TooManyAutomata)
        }

        // Initialise automata
        for i in 0..self.automata.len() {
            let span = span!(Level::INFO, "automata", index = i);
            let _guard = span.enter();

            let a = &mut self.automata[i];

            // Init object
            a.on_init(i as u8, deck)?;

            // Render object
            a.render(i as u8, deck)?;
        }

        let mut last_update = SystemTime::now();

        // Run loop
        loop {
            // Poll for button presses
            let buttons = match deck.read_buttons(Some(*opts.block_period)) {
                Ok(b) => Some(b),
                Err(DeckError::NoData) => None,
                Err(e) => return Err(Error::Deck(e)),
            };

            // Handle button presses
            if let Some(b) = &buttons {
                for i in 0..self.automata.len() {
                    let span = span!(Level::INFO, "automata", index = i);
                    let _guard = span.enter();

                    let a = &mut self.automata[i];

                    if b[i] != 0 {
                        a.on_press(i as u8, deck)?;
                    }
                }
            }

             // Handle periodic updates
             let now = SystemTime::now();
             if now.duration_since(last_update).unwrap() > *opts.poll_period {

                for i in 0..self.automata.len() {
                    let span = span!(Level::INFO, "automata", index = i);
                    let _guard = span.enter();
    
                    let a = &mut self.automata[i];

                    a.on_poll(i as u8, deck)?;

                }
                
                last_update = now;
             }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_automation() {
        let a = Automation::load("example.toml")
            .expect("error loading automation");
    }
}