# Autodeck

A simple automation daemon for linux, designed to interact with Elgato Stream Deck devices. See [example.toml](./example.toml) for a set of example automations, and the [docs](https://docs.rs/autodeck) for further information on automation options.


## Status

WIP, basic functionality working, tested only with a stream deck mini, pull requests welcome.

[![GitHub tag](https://img.shields.io/github/tag/ryankurte/autodeck.svg)](https://github.com/ryankurte/autodeck)
[![Travis Build Status](https://travis-ci.org/ryankurte/autodeck.svg?branch=master)](https://travis-ci.org/ryankurte/autodeck)
[![Crates.io](https://img.shields.io/crates/v/autodeck.svg)](https://crates.io/crates/autodeck)
[![Docs.rs](https://docs.rs/autodeck/badge.svg)](https://docs.rs/autodeck)


## Usage

### Installation

Precompiled binaries are available on the [releases](https://github.com/ryankurte/autodeck/releases) page (including a .deb that will install configuration files and a systemd unit). The utility can also be installed from source using `cargo install autodeck`, but you'll need to manage the configuration files etc. by yourself.

For the debian package, automations are specified in `/etc/autodeck/autodeck.toml` with configuration in `/etc/autodeck/autodeck.env`.

You can register the service at startup with `sudo systemctl enable autodeck`, and check the logs with `sudo journalctl -u autodeck --follow`.


### Automation Files

An automation file consists of a list of `[[automata]]`, each corresponding to a single button on a stream deck.
These `automata` have a `state` field describing their intitial state, as well as an `on_init` executor.

Each `automata` contains a set of named `states` (for example, `[automata.states.error]`), these may contain `on_press` and `on_poll` executors, called when the button is pressed or under periodic polling, as well as a `display` object that defines what will be displayed on the button in a given state.

`Executors` contain instructions about commands that should be executed and how these effect the state of an automata. These contain a `func` argument that is the command to be executed, an `args` list of arguments to be passed to the command (note that args cannot be included in the `func` string directly), a pair of `on_success` and `on_failure` fields mapping to the next states.

In general success is determined by the return code of the command (0 = success, non-0 = error), however, in some instances a command will always return 0 however may not have executed the desired action. In these cases you may use the `success_filter` and `failure_filter` fields that to check for matching data from standard out, and the `on_failure` next state field mapping.



