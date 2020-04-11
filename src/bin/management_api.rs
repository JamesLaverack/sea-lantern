use clap::{App, Arg};


use log::{error, info, debug};

use regex::Regex;

use tonic::{transport::Server, Request, Response, Status};

use management::minecraft_management_server::{MinecraftManagementServer, MinecraftManagement};
use management::{ListPlayersReply, Player};
use std::net::SocketAddr;

use rcon::Connection as RconConnection;


// SaveAllRequest, SaveAllReply, DisableAutomaticSaveRequest, DisableAutomaticSaveReply, EnableAutomaticSaveRequest, EnableAutomaticSaveReply

pub mod management {
    tonic::include_proto!("management");
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
    /*
    async fn disable_automatic_save(
        &self,
        _request: Request<DisableAutomaticSaveRequest>,
    ) -> Result<Response<DisableAutomaticSaveReply>, Status> {
        info!("Got a request to disable automatic save");
        // Register to logs
        let logs = &mut self.logs.lock().await.subscribe();
        // Send the save-all command
        match self.input.lock().await.clone().send("save-off".to_string()).await {
            Ok(_) => (),
            Err(e) => {
                error!("Error sending save-off command: {}", e);
                return Err(Status::unavailable("Failed to communicate with Minecraft process."));
            },
        }
        // Parse response, waiting for up to half a second
        let delay_millis = 500;
        let mut delay = time::delay_for(Duration::from_millis(delay_millis));

        let save_off_regex = Regex::new(r"^\[\d\d:\d\d:\d\d\] \[Server thread/INFO\]: (Automatic saving is now disabled|Saving is already turned off)$").unwrap();

        // Give it a short timeout to start the save, and a long timeout to finish it.
        loop {
            tokio::select! {
                _ = &mut delay => {
                    error!("operation timed out");
                    return Err(Status::deadline_exceeded(format!("Minecraft did not respond in under {}ms", delay_millis)));
                },
                Ok(line) = logs.recv() => {
                    debug!("Save all task, analysing log line: {}", line);
                    if save_off_regex.is_match(&line) {
                        debug!("Minecraft declared save-off.");
                        return Ok(Response::new(DisableAutomaticSaveReply{}));
                    }
                },
            }
        }
    }

    async fn enable_automatic_save(
        &self,
        _request: Request<EnableAutomaticSaveRequest>,
    ) -> Result<Response<EnableAutomaticSaveReply>, Status> {
        info!("Got a request to enable automatic save");
        // Register to logs
        let logs = &mut self.logs.lock().await.subscribe();
        // Send the save-all command
        match self.input.lock().await.clone().send("save-on".to_string()).await {
            Ok(_) => (),
            Err(e) => {
                error!("Error sending save-on command: {}", e);
                return Err(Status::unavailable("Failed to communicate with Minecraft process."));
            },
        }
        // Parse response, waiting for up to half a second
        let delay_millis = 500;
        let mut delay = time::delay_for(Duration::from_millis(delay_millis));

        let save_off_regex = Regex::new(r"^\[\d\d:\d\d:\d\d\] \[Server thread/INFO\]: (Automatic saving is now enabled|Saving is already turned on)$").unwrap();

        // Give it a short timeout to start the save, and a long timeout to finish it.
        loop {
            tokio::select! {
                _ = &mut delay => {
                    error!("operation timed out");
                    return Err(Status::deadline_exceeded(format!("Minecraft did not respond in under {}ms", delay_millis)));
                },
                Ok(line) = logs.recv() => {
                    debug!("Save all task, analysing log line: {}", line);
                    if save_off_regex.is_match(&line) {
                        debug!("Minecraft declared save-off.");
                        return Ok(Response::new(EnableAutomaticSaveReply{}));
                    }
                },
            }
        }
    }

    async fn save_all(
        &self,
        _request: Request<SaveAllRequest>,
    ) -> Result<Response<SaveAllReply>, Status> {
        info!("Got a request to Save All");
        // Register to logs
        let logs = &mut self.logs.lock().await.subscribe();
        // Send the save-all command
        match self.input.lock().await.clone().send("save-all".to_string()).await {
            Ok(_) => (),
            Err(e) => {
                error!("Error sending save-all command: {}", e);
                return Err(Status::unavailable("Failed to communicate with Minecraft process."));
            },
        }
        // Parse response, waiting for up to half a second
        let delay_millis = 500;
        let mut delay = time::delay_for(Duration::from_millis(delay_millis));

        let start_save_regex = Regex::new(r"^\[\d\d:\d\d:\d\d\] \[Server thread/INFO\]: Saving the game \(this may take a moment!\)$").unwrap();
        let end_save_regex = Regex::new(r"^\[\d\d:\d\d:\d\d\] \[Server thread/INFO\]: Saved the game").unwrap();


        // Give it a short timeout to start the save, and a long timeout to finish it.
        loop {
            tokio::select! {
                _ = &mut delay => {
                    error!("operation timed out");
                    return Err(Status::deadline_exceeded(format!("Minecraft did not respond in under {}ms", delay_millis)));
                },
                Ok(line) = logs.recv() => {
                    debug!("Save all task, analysing log line: {}", line);
                    if start_save_regex.is_match(&line) {
                        debug!("Minecraft declared start of save.");
                        break;
                    }
                },
            }
        }

        let full_delay_seconds = 5;
        let mut full_delay = time::delay_for(Duration::from_secs(full_delay_seconds));
        // Second, longer timeout for the full save
        loop {
            tokio::select! {
                _ = &mut full_delay => {
                    error!("operation timed out");
                    return Err(Status::deadline_exceeded(format!("Minecraft did not complete save in under {} seconds", full_delay_seconds)));
                },
                Ok(line) = logs.recv() => {
                    debug!("Save all task, analysing log line: {}", line);
                    if end_save_regex.is_match(&line) {
                        debug!("Minecraft declared end of save.");
                        return Ok(Response::new(SaveAllReply{}));
                    }
                },
            }
        }

    }
    */
    async fn list_players(
        &self,
        _request: Request<()>,
    ) -> Result<Response<ListPlayersReply>, Status> {
        info!("Got a request to list players");
        let mut conn = match RconConnection::connect(self.rcon_address, self.rcon_password.as_str()).await {
            Ok(c) => c,
            Err(e) => {
                error!("Encountered error connecting to server: {:?}", e);
                return Err(Status::unavailable("Unable to connect to Minecraft"));
            },
        };
        let response = match conn.cmd("list uuids").await {
            Ok(c) => c,
            Err(e) => {
                error!("Encountered error sending command to server: {:?}", e);
                return Err(Status::unavailable("Unable to connect to Minecraft"));
            },
        };

        let player_list_regex = Regex::new(r"^There are (?P<current>\d+) of a max (?P<max>\d+) players online:").unwrap();
        let player_details_regex = Regex::new(r"(?P<name>\w+)[ ]\((?P<uuid>[0-9a-fA-F]{8}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{12})\)").unwrap();

        debug!("Got response {}", response);

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
                error!("Response from a 'list uuids' did not match expectation!");
                Err(Status::unknown("Minecraft responded in an unexpected way"))
            },
        };
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