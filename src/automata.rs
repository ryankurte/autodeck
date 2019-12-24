
use std::collections::HashMap;
use std::process::{Command};

use serde::{Serialize, Deserialize};
use streamdeck::{StreamDeck, Colour, ImageOptions};

use crate::Error;

/// Automata
#[derive(Debug, Serialize, Deserialize)]
pub struct Automata {
    /// Initialisation state
    state: String,

    /// Initialisation executor
    on_init: Option<Exec>,

    /// State map
    states: HashMap<String, State>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Init,
    Press,
    Poll,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Output {
    Success,
    Failure,
    Error,
}

impl Automata {

    pub fn get_state(&self) -> Result<&State, Error> {
        match self.states.get(&self.state) {
            Some(s) => Ok(s),
            None => Err(Error::InvalidState),
        }
    }

    pub fn update(&mut self, event: Event, index: u8, deck: &mut StreamDeck) -> Result<(), Error> {

        // Fetch current state
        let state = self.get_state()?;

        // Match event type
        let e = match event {
            Event::Init => self.on_init.as_ref(),
            Event::Press => state.on_press.as_ref(),
            Event::Poll => state.on_poll.as_ref(),
        };

        // Fetch executor
        let e = match e {
            Some(e) => e,
            None => return Ok(()),
        };

        // Run executor
        let res = e.run();

        // Match next state
        let next_state = match (&res, &e.on_success, &e.on_failure, &e.on_error) {
            (Output::Success, Some(s), _, _) => Some(s),
            (Output::Failure, _, Some(s), _) => Some(s),
            (Output::Error,   _, _, Some(s)) => Some(s),
            _ => None,
        };

        info!("result: {:?} current state: {}, new state: {:?}", res, self.state, next_state);

        // Run executor and update state
        if let Some(s) = next_state {
            
            self.state = s.to_string();

            // Render object
            self.render(index, deck)?;
        }
        
        Ok(())
    }

    pub fn on_init(&mut self, index: u8, deck: &mut StreamDeck) -> Result<(), Error> {
        self.update(Event::Init, index, deck)
    }

    pub fn on_press(&mut self, index: u8, deck: &mut StreamDeck) -> Result<(), Error> {
        self.update(Event::Press, index, deck)
    }

    pub fn on_poll(&mut self, index: u8, deck: &mut StreamDeck) -> Result<(), Error> {
        self.update(Event::Poll, index, deck)
    }

    pub fn render(&self, index: u8, deck: &mut StreamDeck) -> Result<(), Error> {
        // Update display for active state
        let state = self.get_state()?;

        if let Some(d) = &state.display {
            d.render(index, deck)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    display: Option<Display>,

    /// On button press action
    on_press: Option<Exec>,

    /// Periodic update function
    on_poll: Option<Exec>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum Display {
    Colour(Colour),
    Image{
        file: String, 

        #[serde(flatten)]
        options: ImageOptions
    },
}

impl Display {
    /// Render display information for the given index on the provided deck
    pub fn render(&self, index: u8, deck: &mut StreamDeck) -> Result<(), Error> {
        debug!("Updating display index: {} config: {:?}", index, self);

        match self {
            Display::Colour(colour) => deck.set_button_rgb(index, colour),
            Display::Image{file, options} => deck.set_button_file(index, file, options),
        }.map_err(Error::Deck)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Exec {
    /// Function to be executed
    func: Option<String>,

    /// Function arguments
    #[serde(default)]
    args: Vec<String>,

    /// String for matching function output success
    /// 
    /// If stdout does not contain this string the command will be rejected as an failure
    success_filter: Option<String>,

    /// String for matching function output errors
    /// 
    /// If stdout contains this string the command will be rejected as an failure
    failure_filter: Option<String>,

    /// State to change to on success
    on_success: Option<String>,

    /// State to change to on failure
    on_failure: Option<String>,
    
    /// State to change to on error
    on_error: Option<String>
}

impl Exec {

    fn run(&self) -> Output {
        // Skip execution if no function provided
        let cmd = match &self.func {
            Some(v) => v,
            None => {
                info!("no function bound, assuming success");
                return Output::Success
            },
        };

        let mut cmd = Command::new(cmd);
        if self.args.len() > 0 {
            cmd.args(&self.args);
        }

        info!("executing command: {:?}", cmd);

        // Execute function if specified
        let res = match cmd.output() {
            Ok(v) => v,
            Err(e) => {
                error!("command error: {:?}", e);
                return Output::Error
            }
        };

        // TODO: make this configurable?
        //std::io::stdout().write_all(&res.stdout).unwrap();
        //std::io::stderr().write_all(&res.stderr).unwrap();

        // Check for success response
        if !res.status.success() {
            info!("command error");
            return Output::Error
        }

        // Attempt to parse out result
        let s = match String::from_utf8(res.stdout) {
            Ok(v) => v,
            Err(e) => {
                error!("error parsing stdout: {:?}", e);
                return Output::Error
            }
        };

        // Run success filter if provided
        if let Some(f) = &self.success_filter {
            if !s.contains(f) {
                info!("command rejected due to filter miss (success: '{}')", f);
                return Output::Failure
            } else {
                debug!("command accepted due to filter match (success: '{}')", f);
            }
        }

        // Run failure filter if provided
        if let Some(f) = &self.failure_filter {
            if s.contains(f) {
                info!("command rejected due to filter match (failure: '{}')", f);
                return Output::Failure
            } else {
                debug!("command accepted due to filter miss (failure: '{}')", f);
            }
        }

        // Return ok on success
        info!("command ok");
        Output::Success
    }
}