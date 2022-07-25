use std::{collections::HashMap, sync::{Arc, Mutex}, fs::File};
use colored::Colorize;
use enigo::{Enigo, KeyboardControllable};
use serde_yaml::Value;
use tokio::{net::TcpListener, io::{BufReader, AsyncBufReadExt, AsyncWriteExt}};

fn error(message: String) {
    println!("{}{}", "ERROR: ".red(), message.red());
}

fn get_key(text: &str) -> enigo::Key {
    match text {
        "left" => enigo::Key::LeftArrow,
        "right" => enigo::Key::RightArrow,
        "up" => enigo::Key::UpArrow,
        "down" => enigo::Key::DownArrow,
        _ => enigo::Key::Layout(text.chars().next().unwrap())
    }
}

#[tokio::main]
async fn main() {
    // Initialize enigo
    // let enigo = Arc::new(Mutex::new(Enigo::new()));

    // Read yaml config
    let file = File::open("config.yaml").expect("Could not load file!");
    let keymaps: Value = serde_yaml::from_reader(file).unwrap();
    // Create users map
    let users = Arc::new(Mutex::new(HashMap::new()));

    // Start socket server
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let users = users.clone();
        let keymaps = keymaps.clone();
        // let enigo = enigo.clone();

        tokio::spawn(async move {

            let (read, mut writer) = socket.split();
            let mut reader = BufReader::new(read);
            let mut message = String::new();

            // handshake
            let bytes_read = reader.read_line(&mut message).await.unwrap();
            if bytes_read == 0 || message.trim_end() != "handshake" {
                error("error in hanshake".to_string());
                return;
            } else {
                let info = keymaps.as_mapping().unwrap().iter()
                    .fold(String::new(), |s, k| {
                        let name = k.0.as_str().unwrap().to_string();
                        if s.len() == 0 { name } else { s + "," + &name }
                    });

                println!("Client connected: {}", addr);
                writer.write_all(info.as_bytes()).await.expect("Error writing");
            }

            loop {
                let bytes_read = reader.read_line(&mut message).await.unwrap();
                if bytes_read == 0 {
                    println!("Client disconnected {}", addr);
                    break;
                }

                let split = message.split("::");
                let vec = split.collect::<Vec<&str>>();
                if vec.len() != 2 {
                    error("Message should be like 'command::value'".to_string());
                    message.clear();
                    continue;
                }

                let mut map = users.lock().unwrap();
                let value = vec[1].trim_end().to_string();
                match vec[0] {
                    "use" => {
                        if keymaps.get(&value).is_some() {
                            map.insert(addr.to_string(), value);
                            println!("map -> {:?}", map);
                        } else {
                            error("El perfil es invalido (no esta en el archivo .yaml)".to_string());
                        }
                    }
                    "press" => {
                        map.get(&addr.to_string()).map(|mapping| {
                            keymaps[mapping][value].as_str().map(|key| {
                                println!("pressing {}", key);
                                let mut enigo = Enigo::new();
                                // let enigo = enigo.lock().unwrap();
                                enigo.key_down(get_key(key));
                            });
                        }).or_else(|| Some(error("La direccion no tiene un perfil asignado".to_string())));
                    }
                    _ => error("Command not recognized".to_string())
                }
                message.clear();
            }
        });
    }
}
