use clap::{App, Arg};
use std::path::PathBuf;
use log::{debug, error, info};
use ini::Ini;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let matches = App::new("Sea Lantern Server Properties Updater")
        .version("0.1.0")
        .author("James Laverack <james@jameslaverack.com>")
        .about("Updates the server.properties file before Minecraft starts")
        .arg(
            Arg::with_name("server-root")
                .long("server-root")
                .takes_value(true)
                .required(true)
                .help("Filepath to the server root."),
        )
        .get_matches();

    match update_properties_file(matches.value_of("server-root").unwrap()) {
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

fn update_properties_file<P: Into<PathBuf>>(server_root: P) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut server_properties_file_path = server_root.into();
    server_properties_file_path.push("server.properties");

    let mut i = if server_properties_file_path.exists() {
        Ini::load_from_file(server_properties_file_path.clone())?
    } else {
        Ini::new()
    };

    // TODO Don't hardcode all of this
    i.set_to::<String>(Option::None, "enable-rcon".to_string(), "true".to_string());
    i.set_to::<String>(Option::None, "rcon.port".to_string(), "25575".to_string());
    i.set_to::<String>(Option::None, "rcon.password".to_string(), "password".to_string());
    i.set_to::<String>(Option::None, "broadcast-rcon-to-ops".to_string(), "true".to_string());

    i.write_to_file(server_properties_file_path.clone())?;
    Ok(server_properties_file_path.to_path_buf())
}
