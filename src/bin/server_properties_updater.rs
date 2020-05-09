use clap::{App, Arg};
use std::path::PathBuf;
use log::{debug, error, info};
use ini::Ini;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // These are the names of properties in a minecraft server.properties file. It is not for this
    // component to parse and validate these, just write them in. The ones below are supported as
    // flags.
    let properties = [
        "motd",
        "difficulty",
        "gamemode",
        "view-distance",
        "max-world-size",
        "max-build-height",
        "max-players",
        "pvp",
        "level-seed",
        "level-type",
        "spawn-animals",
        "spawn-monsters",
        "spawn-npcs",
    ];

    // Put "setting-" on the front for the actual flag. For example, "max-players" becomes the long
    // name "setting-max-players" which will be used as a flag "--setting-max-players".
    let long_names: Vec<String>  = properties.iter()
        .map(|prop| format!("setting-{}", prop))
        //.map(|l| l.as_str())
        .collect();

    let mut app = App::new("Sea Lantern Server Properties Updater")
        .version("0.1.0")
        .author("James Laverack <james@jameslaverack.com>")
        .about("Updates the server.properties file before Minecraft starts")
        .arg(
            Arg::with_name("server-root")
                .long("server-root")
                .takes_value(true)
                .required(true)
                .help("Filepath to the server root."))
        .arg(Arg::with_name("path-to-rcon-password")
            .long("path-to-rcon-password")
            .takes_value(true)
            .help("Filepath to the rcon password file on disk"));

    // Fold over the app to inject an argument for each property
    app = properties.iter()
        .zip(long_names.iter())
        .fold(app, |a, (prop, long_name)| {
            a.arg(Arg::with_name(prop)
                    .long(long_name.as_str())
                    .takes_value(true))
        });

    let matches = app.get_matches();

    match update_properties_file(
        matches.value_of("server-root").unwrap(),
        |i|{

            let mut set_value = |key: &str, value: &str| {
                i.set_to::<String>(Option::None, key.to_string(), value.to_string());
            };

            // We always want rcon enabled
            set_value("enable-rcon", "true");

            // RCON password is also special
            // TODO: Read this from a file
            set_value("rcon.password", "12345");

            properties.iter().for_each(|prop| {
                match matches.value_of(prop) {
                    Some(val) => {
                        info!("Setting property {} to: {}", prop, val);
                        set_value(prop, val);
                    },
                    None => (),
                }
            });
    }) {
        Ok(path) => {
            // We've written the file, but the path we wrote too might not be valid UTF-8.
            match path.to_str() {
                Some(path_str) => {
                    info!("Updated server properties file at {:?}", path_str);
                },
                None => {
                    debug!("Path wasn't UTF-8, unable to log path");
                    info!("Updated server properties file");
                },
            }
        },
        Err(why) => {
            error!("Failed to update server properties file: {:?}", why)
        }
    }
}

fn update_properties_file<P: Into<PathBuf>, F: Fn(&mut Ini)>
(server_root: P, update_file: F) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut server_properties_file_path = server_root.into();
    server_properties_file_path.push("server.properties");

    let mut i = if server_properties_file_path.exists() {
        Ini::load_from_file(server_properties_file_path.clone())?
    } else {
        Ini::new()
    };

    update_file(&mut i);

    i.write_to_file(server_properties_file_path.clone())?;
    Ok(server_properties_file_path.to_path_buf())
}
