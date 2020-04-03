use clap::{App, Arg};

use std::sync::{Arc};

use log::{error, info, debug, trace};

use regex::Regex;

use tokio::io::AsyncBufReadExt;
use tokio::prelude::*;
use tokio::net::UnixStream;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::io::BufReader;
use tokio::time::{self, Duration};

use tonic::{transport::Server, Request, Response, Status};

use management::minecraft_management_server::{MinecraftManagementServer, MinecraftManagement};
use management::{ListUsersRequest, ListUsersReply};

pub mod management {
    tonic::include_proto!("management");
}

#[derive(Debug)]
pub struct DummyMinecraftManagement {
    logs: Arc<Mutex<tokio::sync::broadcast::Sender<String>>>,
    input: Arc<Mutex<tokio::sync::mpsc::Sender<String>>>,
}

#[tonic::async_trait]
impl MinecraftManagement for DummyMinecraftManagement {
    async fn list_users(
        &self,
        _request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersReply>, Status> {
        info!("Got a request to list users");
        // Register to logs
        let logs = &mut self.logs.lock().await.subscribe();
        // Send the list uuid command
        match self.input.lock().await.clone().send("list uuids".to_string()).await {
            Ok(_) => (),
            Err(e) => {
                error!("Error sending list users command: {}", e);
                return Err(Status::unavailable("Failed to communicate with Minecraft process."));
            },
        }
        // Parse response, waiting for up to half a second
        let delay_millis = 500;
        let mut delay = time::delay_for(Duration::from_millis(delay_millis));

        // Of course, we're just looking for the first log line that looks like the response. If
        // multiple lists were executed at the same time, Minecraft would respond to all of them
        // and we might see a log line intended for another request. That's acceptable however as
        // it would still be correct.

        // Match from the start of the line so you can't fake it with a chat comment
        let player_list_regex = Regex::new(r"^\[\d\d:\d\d:\d\d\] \[Server thread/INFO\]: There are (?P<current>\d+) of a max (?P<max>\d+) players online:").unwrap();

        loop {
            tokio::select! {
                _ = &mut delay => {
                    error!("operation timed out");
                    return Err(Status::deadline_exceeded(format!("Minecraft did not respond in under {}ms", delay_millis)));
                },
                Ok(line) = logs.recv() => {
                    debug!("List users task, analysing log line: {}", line);
                    match player_list_regex.captures(&line) {
                        Some(caps) => {
                            let reply = management::ListUsersReply {
                                // If the regex matched, these two capture groups must be `\d+`, so
                                // they're ints. I guess there's a tiny chance that it's too big to
                                // fit into an u32, but that would be super weird anyway.
                                online_players: caps["current"].parse::<u32>().unwrap(),
                                max_players: caps["max"].parse::<u32>().unwrap(),
                                // TODO
                                users: [].to_vec(),
                            };

                            return Ok(Response::new(reply))
                        },
                        None => {()},
                    };
                },
            }
        }

    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let matches = App::new("Sea Lantern Management API")
        .version("0.1.0")
        .author("James Laverack <james@jameslaverack.com>")
        .about("Connects to a Minecraft server run using the runtime over a UNIX socket, provides a gRPC API.")
        .arg(Arg::with_name("port")
            .long("grpc-port")
            .required(true)
            .takes_value(true)
            .help("Port to expose gRPC API on"))
        .arg(Arg::with_name("socket")
            .long("socket")
            .required(true)
            .takes_value(true)
            .help("URL of Socket to read/write from"))
        .get_matches();

    let addr = format!("[::1]:{}", matches.value_of("port").unwrap()).parse()?;


    let (minecraft_stdin_mpsc, mut minecraft_stdin_mpsc_output) = mpsc::channel(100);


    // We'll have a short-lived Tokio task for each gRPC connection. Use a broadcast channel to
    // stream logs to the tasks, and a mpsc channel to take commands.
    let (logs_broadcast, _) = broadcast::channel(16);

    let logs_arc = Arc::new(Mutex::new(logs_broadcast));

    let minecraft_management = DummyMinecraftManagement{
        input: Arc::new(Mutex::new(minecraft_stdin_mpsc)),
        logs: logs_arc.clone(),
    };

    tokio::spawn(async move {
        let socket_address = matches.value_of("socket").unwrap();
        info!("Connecting to Minecraft process on socket '{}'", socket_address);
        let mut socket = match UnixStream::connect(socket_address).await {
            Ok(s) => s,
            Err(err) => {
                error!("Failed to connect to socket: {}", err);
                panic!(err);
            },
        };
        let (unbuffered_logs, mut write) = socket.split();
        let mut buffered_logs = BufReader::new(unbuffered_logs).lines();

        loop {
            tokio::select! {
                Some(line) = minecraft_stdin_mpsc_output.recv() => {
                    debug!("Got command from task, sending to runtime: {}", line);
                    match write.write_all(format!("{}\n", line).as_bytes()).await {
                        Ok(_) => (),
                        Err(e) => error!("Failed to send to runtime: {:?}", e),
                    };
                },
                Ok(Some(line)) = buffered_logs.next_line() => {
                    debug!("Saw log line from runtime, broadcasting to tasks: {}", line);
                    match logs_arc.lock().await.send(line) {
                        Ok(_) => (),
                        Err(_) => trace!("Failed to broadcast to tasks, probably because there aren't any."),
                    };
                },
            };
        }
    });

    Server::builder()
        .add_service(MinecraftManagementServer::new(minecraft_management))
        .serve(addr)
        .await?;

    Ok(())
}