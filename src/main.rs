extern crate swayipc;
use swayipc::reply::Event;
use swayipc::{Connection, EventType};

extern crate swaywsr;

extern crate exitfailure;
use exitfailure::ExitFailure;

#[macro_use]
extern crate clap;
use clap::{App, Arg};

fn main() -> Result<(), ExitFailure> {
    let matches = App::new("swaywsr - sway workspace renamer")
        .version(crate_version!())
        .author(crate_authors!(",\n"))
        .arg(
            Arg::with_name("icons")
                .long("icons")
                .help("Sets icons to be used")
                .possible_values(&["awesome"])
                .takes_value(true),
        )
        .arg(
            Arg::with_name("no-names")
                .long("no-names")
                .help("Set to no to display only icons (if available)"),
        )
        .arg(
            Arg::with_name("no-dupes")
                .long("no-dupes")
                .help("Set to display each instance of a class only once"),
        )
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .help("Path to toml config file")
                .takes_value(true),
        )
        .get_matches();

    let icons = matches.value_of("icons").unwrap_or("");
    let no_names = matches.is_present("no-names");
    let no_dupes = matches.is_present("no-dupes");
    let options = match matches.value_of("config") {
        Some(filename) => {
            let file_config = match swaywsr::config::read_toml_config(filename) {
                Ok(config) => config,
                Err(e) => panic!("Could not parse config file\n {}", e),
            };
            swaywsr::Options {
                icons: file_config
                    .icons
                    .into_iter()
                    .chain(swaywsr::icons::get_icons(&icons))
                    .collect(),
                aliases: file_config.aliases,
                general: file_config.general,
                names: !no_names,
                no_dupes: no_dupes,
            }
        }
        None => swaywsr::Options {
            icons: swaywsr::icons::get_icons(&icons),
            aliases: swaywsr::config::EMPTY_MAP.clone(),
            general: swaywsr::config::EMPTY_MAP.clone(),
            names: !no_names,
            no_dupes: no_dupes,
        },
    };

    let subs = [EventType::Window, EventType::Workspace];
    let connection = Connection::new()?;
    let mut command_connection = Connection::new()?;

    swaywsr::update_tree(&mut command_connection, &options)?;

    for event in connection.subscribe(&subs)? {
        match event? {
            Event::Window(e) => {
                if let Err(error) =
                    swaywsr::handle_window_event(&e, &mut command_connection, &options)
                {
                    eprintln!("handle_window_event error: {}", error);
                }
            }
            Event::Workspace(e) => {
                if let Err(error) =
                    swaywsr::handle_workspace_event(&e, &mut command_connection, &options)
                {
                    eprintln!("handle_workspace_event error: {}", error);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
