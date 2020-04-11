use clap::{App, Arg};


use log::{error, info, debug};

use regex::Regex;

use tonic::{transport::Server, Request, Response, Status};

use management::minecraft_management_server::{MinecraftManagementServer, MinecraftManagement};
use management::{ListPlayersReply, Player};
use std::net::SocketAddr;

use rcon::{Connection as RconConnection, Error as RconError};

pub mod management {
    tonic::include_proto!("management");
}

fn convert_conn_err<T>(r: Result<T, RconError>) -> Result<T, Status> {
    return match r {
        Ok(c) => Ok(c),
        Err(e) => {
            error!("Encountered error communicating with Minecraft: {:?}", e);
            Err(Status::unavailable("Unable to connect to Minecraft"))
        },
    }
}

#[derive(Debug)]
pub struct RconMinecraftManagement {
    rcon_address: SocketAddr,
    rcon_password: String,
}

impl RconMinecraftManagement {
    fn new<S, A>(rcon_address: A, rcon_password: S) -> Self where S: Into<String>, A: Into<SocketAddr> {
        RconMinecraftManagement {
            rcon_address: rcon_address.into(),
            rcon_password: rcon_password.into(),
        }
    }

}

#[tonic::async_trait]
impl MinecraftManagement for RconMinecraftManagement {

    async fn list_players(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListPlayersReply>, Status> {
        info!("Got a request to list players");
        const CMD: &str = "save-all";
        let mut conn = convert_conn_err(RconConnection::connect(self.rcon_address, self.rcon_password.as_str()).await)?;
        let response = convert_conn_err(conn.cmd(CMD).await)?;

        let player_list_regex = Regex::new(r"^There are (?P<current>\d+) of a max (?P<max>\d+) players online:").unwrap();
        let player_details_regex = Regex::new(r"(?P<name>\w+)[ ]\((?P<uuid>[0-9a-fA-F]{8}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{12})\)").unwrap();

        debug!("Got response from {}: {}", CMD, response);

        return match player_list_regex.captures(&response) {
            Some(caps) => {
                Ok(Response::new(management::ListPlayersReply {
                    // If the regex matched, these two capture groups must be `\d+`, so
                    // they're unsigned ints. I guess there's a tiny chance that it's too big to
                    // fit into an u32, but that would be super weird anyway.
                    online_players: caps["current"].parse::<u32>().unwrap(),
                    max_players: caps["max"].parse::<u32>().unwrap(),
                    players: player_details_regex
                        .captures_iter(&response)
                        .map(|m| {
                            Player {
                                name: m["name"].to_string(),
                                uuid: m["uuid"].to_string(),
                            }
                        })
                        .collect(),
                }))
            },
            None => {
                error!("Response from a '{}' did not match expectation!", CMD);
                Err(Status::unknown("Minecraft responded in an unexpected way"))
            },
        };
    }

    async fn enable_automatic_save(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        debug!("Got a request to enable automatic saving");
        const CMD: &str = "save-on";
        let mut conn = convert_conn_err(RconConnection::connect(self.rcon_address, self.rcon_password.as_str()).await)?;
        let response = convert_conn_err(conn.cmd(CMD).await)?;

        let save_on_regex = Regex::new(r"(Automatic saving is now enabled|Saving is already turned on)").unwrap();

        debug!("Got response from {}: {}", CMD, response);

        if save_on_regex.is_match(&response) {
            Ok(Response::new(()))
        } else {
            error!("Response from a '{}' did not match expectation!", CMD);
            Err(Status::unknown("Minecraft responded in an unexpected way"))
        }
    }

    async fn disable_automatic_save(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        debug!("Got a request to disable automatic saving");
        const CMD: &str = "save-off";
        let mut conn = convert_conn_err(RconConnection::connect(self.rcon_address, self.rcon_password.as_str()).await)?;
        let response = convert_conn_err(conn.cmd(CMD).await)?;

        let save_off_regex = Regex::new(r"(Automatic saving is now disabled|Saving is already turned off)").unwrap();

        debug!("Got response from {}: {}", CMD, response);

        if save_off_regex.is_match(&response) {
            Ok(Response::new(()))
        } else {
            error!("Response from a '{}' did not match expectation!", CMD);
            Err(Status::unknown("Minecraft responded in an unexpected way"))
        }
    }

    async fn save_all(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        debug!("Got a request to save all");
        const CMD: &str = "save-all";
        let mut conn = convert_conn_err(RconConnection::connect(self.rcon_address, self.rcon_password.as_str()).await)?;
        let response = convert_conn_err(conn.cmd(CMD).await)?;

        let saved_game_regex = Regex::new(r"Saved the game").unwrap();

        debug!("Got response from {}: {}", CMD, response);

        if saved_game_regex.is_match(&response) {
            Ok(Response::new(()))
        } else {
            error!("Response from a '{}' did not match expectation!", CMD);
            Err(Status::unknown("Minecraft responded in an unexpected way"))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let matches = App::new("Sea Lantern Management API")
        .version("0.1.0")
        .author("James Laverack <james@jameslaverack.com>")
        .about("Connects to a Minecraft server using RCON, provides a gRPC API.")
        .arg(Arg::with_name("grpc-port")
            .long("grpc-port")
            .required(true)
            .takes_value(true)
            .help("Port to expose gRPC API on"))
        .arg(Arg::with_name("minecraft-rcon-address")
            .long("minecraft-rcon-address")
            .required(true)
            .takes_value(true)
            .help("Address of the minecraft server"))
        .arg(Arg::with_name("minecraft-rcon-password")
            .long("minecraft-rcon-password")
            .required(true)
            .takes_value(true)
            .help("Password of the minecraft server"))
        .get_matches();

    let addr = format!("[::1]:{}", matches.value_of("grpc-port").unwrap()).parse()?;
    let minecraft_addr = match matches.value_of("minecraft-rcon-address").unwrap().parse::<SocketAddr>() {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to parse server address {} : {:?}", matches.value_of("minecraft-rcon-address").unwrap(), e);
            return Err(e)?;
        },
    };

    let rcon_server = RconMinecraftManagement::new(
        minecraft_addr,
        matches.value_of("minecraft-rcon-password").unwrap(),
    );

    info!("Serving gRPC API on {:?}", addr);

    Server::builder()
        .add_service(MinecraftManagementServer::new(rcon_server))
        .serve(addr)
        .await?;

    Ok(())
}