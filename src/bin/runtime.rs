use clap::{App, Arg};

use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::net::UnixListener;
use tokio::prelude::*;
use tokio::sync::broadcast;
use tokio::stream::StreamExt;

use std::process::Stdio;

#[tokio::main]
async fn main() -> io::Result<()> {
    let matches = App::new("Sea Lantern Runtime")
        .version("0.1.0")
        .author("James Laverack <james@jameslaverack.com>")
        .about("Runs Minecraft, redirecting its input and output to a socket while preserving writing to real STDOUT.")
        .arg(Arg::with_name("java")
                 .long("java-executable")
                 .required(true)
                 .takes_value(true)
                 .help("Path to the Java executable"))
        .arg(Arg::with_name("server-jar")
                 .long("server-jar")
                 .takes_value(true)
                 .required(true)
                 .help("Path to the Minecraft server JAR"))
        .arg(Arg::with_name("workdir")
                 .long("working-directory")
                 .required(true)
                 .takes_value(true)
                 .help("Working directory when executing Java"))
        .arg(Arg::with_name("socket")
                 .long("socket")
                 .required(true)
                 .takes_value(true)
                 .help("URL of Socket to read/write from"))
        .get_matches();

    // Create Minecraft process, a Rust future actually.
    let mut minecraft_child_process = Command::new(matches.value_of("java").unwrap())
        .arg("-jar")
        .arg(matches.value_of("server-jar").unwrap())
        .arg("--nogui")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .current_dir(matches.value_of("workdir").unwrap())
        .spawn()?;

    let minecraft_stdout = minecraft_child_process.stdout.take()
        .expect("child did not have a handle to stdout");

    let mut minecraft_buffered_stdout = tokio::io::BufReader::new(minecraft_stdout).lines();
    let (logs_broadcast, mut broadcast_out) = broadcast::channel(16);
    let logs_broadcast_clone = logs_broadcast.clone();
    tokio::spawn(async move {
        // Tokio task to copy from process' STDOUT to the broadcast channel
        while let Ok(Some(line)) = minecraft_buffered_stdout.next_line().await {
            match logs_broadcast.send(line) {
                Ok(_) => (),
                Err(_) => (),
            };
        };
    });
    tokio::spawn(async move {
        // Tokio task to copy from broadcast channel to *our* STDOUT
        while let Ok(line) = broadcast_out.recv().await {
            match tokio::io::stdout().write_all(format!("{}\n", line).as_bytes()).await {
                Ok(_) => (),
                Err(_) => (),
            };
        };
    });

    // Await on the process Future in a Tokio task, which actually runs it.
    tokio::spawn(async {
        minecraft_child_process.await
    });


    // Listen on the UNIX socket
    let socket_addr = matches.value_of("socket").unwrap();
    let mut listener = UnixListener::bind(socket_addr).unwrap();

    // Here we convert the `TcpListener` to a stream of incoming connections
    // with the `incoming` method.
    let server = async move {
        let mut incoming = listener.incoming();
        while let Some(socket_res) = incoming.next().await {
            match socket_res {
                Ok(mut socket) => {
                    //let (mut _reader, mut writer) = socket.split();
                    // Spawn a new async task to subscribe to the broadcast and copy into the
                    // socket.
                    let mut broadcast_out_for_socket = logs_broadcast_clone.subscribe();
                    tokio::spawn(async move {
                        while let Ok(line) = broadcast_out_for_socket.recv().await {
                            match socket.write_all(format!("{}\n", line).as_bytes()).await {
                                Ok(_) => (),
                                Err(_) => (),
                            };
                        };
                    });
                },
                Err(_) => {
                    // Not much to do here. We can't really log out or we'll pollute the Minecraft
                    // logs.
                },
            };
        }
    };

    // Start the server and block this async fn until `server` spins down.
    server.await;
    Ok(())
}
