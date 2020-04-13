use clap::{App, Arg};
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use log::{debug, error, info};


#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let matches = App::new("Sea Lantern EULA Writer")
        .version("0.1.0")
        .author("James Laverack <james@jameslaverack.com>")
        .about("Writes the eula.txt file *if* the user has agreed to the Minecraft EULA.")
        .arg(
            Arg::with_name("agree-minecraft-eula")
                .long("agree-minecraft-eula")
                .help("Set if the eula has been agreed."),
        )
        .arg(
            Arg::with_name("server-root")
                .long("server-root")
                .takes_value(true)
                .required(true)
                .help("Filepath to the server root."),
        )
        .get_matches();

    if matches.is_present("agree-minecraft-eula") {
        match write_eula_file(matches.value_of("server-root").unwrap()) {
            Ok(path) => {
                // We've written the file, but the path we wrote too might not be valid UTF-8.
                match path.to_str() {
                    Some(path_str) => {
                        info!("Written EULA file to {:?}", path_str);
                    },
                    None => {
                        debug!("Path wasn't UTF-8, unable to log path");
                        info!("Written EULA file");
                    },
                }
            },
            Err(why) => {
                error!("Failed to write EULA file: {:?}", why)
            }
        }
    } else {
        error!("EULA not agreed to.");
    }
}

fn write_eula_file<P: Into<PathBuf>>(server_root: P) -> Result<PathBuf, std::io::Error> {
    let mut eula_file_path = server_root.into();
    eula_file_path.push("eula.txt");

    let mut file = File::create(eula_file_path.clone())?;
    file.write_all("eula=true".as_bytes())?;

    Ok(eula_file_path.to_path_buf())
}
