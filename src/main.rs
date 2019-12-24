
#[macro_use]
extern crate tracing;

extern crate tracing_subscriber;
use tracing_subscriber::{FmtSubscriber};
use tracing_subscriber::filter::{LevelFilter};

extern crate structopt;
use structopt::StructOpt;

extern crate streamdeck;
use streamdeck::{StreamDeck, Filter};

extern crate autodeck;
use autodeck::{Automation, Options};

#[derive(StructOpt)]
#[structopt(name = "autodeck", about = "An Elgato StreamDeck based automation daemon")]
struct CliOptions {

    #[structopt(short, long, default_value="/etc/autodeck/autodeck.toml")]
    /// Automation definition file
    config_file: String,

    #[structopt(flatten)]
    filter: Filter,

    #[structopt(flatten)]
    automate: Options,

    #[structopt(long = "log-level", default_value = "info")]
    /// Enable verbose logging
    level: LevelFilter,
    
}

fn main() {
    // Parse options
    let opts = CliOptions::from_args();

    // Initialise logging
    //let env_filter = EnvFilter::from_default_env().add_directive("streamdeck=info".parse().unwrap());
    let _ = FmtSubscriber::builder().with_max_level(opts.level.clone()).try_init();

    // Load configuration file
    let mut a = match Automation::load(&opts.config_file) {
        Ok(v) => v,
        Err(e) => {
            error!("Error loading automation: {:?}", e);
            std::process::exit(-1);
        }
    };

    // Connect to device
    let mut deck = match StreamDeck::connect(opts.filter.vid, opts.filter.pid, opts.filter.serial) {
        Ok(d) => d,
        Err(e) => {
            error!("Error connecting to streamdeck: {:?}", e);
            std::process::exit(-2);
        }
    };
    
    // Run engine
    if let Err(e) = a.run(&mut deck, opts.automate) {
        error!("Automata error: {:?}", e);
        std::process::exit(-23);
    }
}
