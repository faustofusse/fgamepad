use std::{collections::HashMap, sync::{Mutex, Arc}};
use colored::Colorize;
use serde_yaml::Value;
use std::fs::File;
use tokio::{net::TcpListener, io::{BufReader, AsyncBufReadExt}};

fn error(message: String) {
    println!("{}{}", "ERROR: ".red(), message.red());
}

#[tokio::main]
async fn main() {
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
        tokio::spawn(async move {

            let (read, _writer) = socket.split();
            let mut reader = BufReader::new(read);
            let mut message = String::new();

            loop {
                let bytes_read = reader.read_line(&mut message).await.unwrap();
                if bytes_read == 0 {
                    error("Message is empty!".to_string());
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
                            println!("Se seteo el custom {} a la direccion {}", value, addr.to_string());
                            map.insert(addr.to_string(), value);
                            println!("El mapa hasta ahora: {:?}", map);
                        } else {
                            error("El mapeo es invalido (no esta en el archivo .yaml)".to_string());
                        }
                    }
                    "press" => {
                        if map.contains_key(&addr.to_string()) {
                            let user_mapping = &map[&addr.to_string()];
                            keymaps.get(user_mapping).map(|mapping| {
                                mapping.get(value).map(|key| {
                                    println!("Pressing key: {}", key.as_str().unwrap_or("-"));
                                }).or_else(|| Some(error("Se recibio una tecla que no esta configurada".to_string())));
                            });
                        } else {
                            error("La direccion no tiene un keymap asignado".to_string());
                        }
                    }
                    _ => error("Command not recognized".to_string())
                }
                message.clear();
            }
        });
    }
}
