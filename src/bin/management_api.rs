use clap::{App, Arg};

use std::sync::{Arc};
use std::fmt;
use std::error;

use log::{error, info, debug, trace};

use regex::Regex;

use tokio::io::AsyncBufReadExt;
use tokio::prelude::*;
use tokio::net::UnixStream;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::io::BufReader;
use tokio::time::{self, Duration};

use tonic::{transport::Server, Request, Response, Status};

use sea_lantern::rcon::{write_rcon_packet, RconPacketType};

use management::minecraft_management_server::{MinecraftManagementServer, MinecraftManagement};
use management::{ListPlayersReply, Player};

// SaveAllRequest, SaveAllReply, DisableAutomaticSaveRequest, DisableAutomaticSaveReply, EnableAutomaticSaveRequest, EnableAutomaticSaveReply

pub mod management {
    tonic::include_proto!("management");
}

#[derive(Debug)]
pub struct RconMinecraftManagement {
    rcon_host :String,
    rcon_port :u16,
}

impl RconMinecraftManagement {
    fn new<S, P>(rcon_host :S, rcon_port :P) -> Self where S: Into<String>, P: Into<u16> {
        RconMinecraftManagement {
            rcon_host: rcon_host.into(),
            rcon_port: rcon_port.into(),
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
        

        let remote_addr: SocketAddr = format!("{}:{}", self.rcon_host, self.rcon_port).parse()?;

        // We use port 0 to let the operating system allocate an available port for us.
        let local_addr: SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
            .parse()?;

        let mut socket = UdpSocket::bind(local_addr).await?;
        const MAX_DATAGRAM_SIZE: usize = 65_507;
        socket.connect(&remote_addr).await?;

        // Auth
        let mut rcon_packet = [0u8; sea_lantern::rcon::MAXIMUM_PAYLOAD_LENGTH];
        rcon_packet_size = write_rcon_packet(0, RconPacketType::Login, &self.rcon_password, &rcon_packet)?;
        socket.send(&rcon_packet[0..rcon_packet_size]).await?;

        let mut data = [0u8; MAX_DATAGRAM_SIZE];
        let len = socket.recv(&mut data).await?;
        println!(
            "Received {} bytes:\n{}",
            len,
            String::from_utf8_lossy(&data[..len])
        );

        Ok(())
        // Auth to rcons
        Ok(Response::new(management::ListPlayersReply {
            // If the regex matched, these two capture groups must be `\d+`, so
            // they're ints. I guess there's a tiny chance that it's too big to
            // fit into an u32, but that would be super weird anyway.
            online_players: 3,
            max_players: 22,
            players: [].to_vec(),
        }))
        /*
        // Parse response, waiting for up to half a second
        let delay_millis = 500;
        let mut delay = time::delay_for(Duration::from_millis(delay_millis));

        // Of course, we're just looking for the first log line that looks like the response. If
        // multiple lists were executed at the same time, Minecraft would respond to all of them
        // and we might see a log line intended for another request. That's acceptable however as
        // it would still be correct.

        // Match from the start of the line so you can't fake it with a chat comment
        let player_list_regex = Regex::new(r"^\[\d\d:\d\d:\d\d\] \[Server thread/INFO\]: There are (?P<current>\d+) of a max (?P<max>\d+) players online:").unwrap();
        let player_details_regex = Regex::new(r"(?P<name>\w+)[ ]\((?P<uuid>[0-9a-fA-F]{8}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{12})\)").unwrap();

        loop {
            tokio::select! {
                _ = &mut delay => {
                    error!("operation timed out");
                    return Err(Status::deadline_exceeded(format!("Minecraft did not respond in under {}ms", delay_millis)));
                },
                Ok(line) = logs.recv() => {
                    debug!("List players task, analysing log line: {}", line);
                    match player_list_regex.captures(&line) {
                        Some(caps) => {
                            let u = player_details_regex
                                .captures_iter(&line)
                                .map(|m| {
                                    Player{
                                        name: m["name"].to_string(),
                                        uuid: m["uuid"].to_string(),
                                    }
                                })
                                .collect();

                            let reply = management::ListPlayersReply {
                                // If the regex matched, these two capture groups must be `\d+`, so
                                // they're ints. I guess there's a tiny chance that it's too big to
                                // fit into an u32, but that would be super weird anyway.
                                online_players: caps["current"].parse::<u32>().unwrap(),
                                max_players: caps["max"].parse::<u32>().unwrap(),
                                // TODO
                                players: u,
                            };

                            return Ok(Response::new(reply))
                        },
                        None => {()},
                    };
                },
            }

        }
        */

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
        .arg(Arg::with_name("minecraft-rcon-host")
            .long("minecraft-rcon-host")
            .required(true)
            .takes_value(true)
            .help("Hostname of the minecraft server"))
        .arg(Arg::with_name("minecraft-rcon-port")
            .long("minecraft-rcon-port")
            .required(true)
            .takes_value(true)
            .help("Port of the minecraft server"))
        .get_matches();

    let addr = format!("[::1]:{}", matches.value_of("grpc-port").unwrap()).parse()?;

    let rcon_server = RconMinecraftManagement::new(
        matches.value_of("minecraft-rcon-host").unwrap(),
        matches.value_of("minecraft-rcon-port").unwrap().parse::<u16>()?,
    );

    Server::builder()
        .add_service(MinecraftManagementServer::new(rcon_server))
        .serve(addr)
        .await?;

    Ok(())
}