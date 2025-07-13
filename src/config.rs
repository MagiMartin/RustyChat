use std::{collections::HashMap, fs, path::Path};
use serde::Deserialize;
use crate::app::App;
use crate::stream::{ServerId, NetEvent};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use crate::app::ServerData;
use crate::app::ChannelData;

// Root struct, all sections optional
#[derive(Debug, Deserialize)]
struct Config {
    config: Option<ClientConfig>,
    theme: Option<Theme>,
    twitch: Option<Twitch>,
    autojoin: Option<AutoJoin>,
}

#[derive(Debug, Deserialize)]
struct ClientConfig {
    nick: String,
}

#[derive(Debug, Deserialize)]
struct Theme {
    fg: Vec<u8>,
    bg: Vec<u8>,
    notification: Vec<u8>,
    highlight: Vec<u8>,
    text: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct Twitch {
    nick: String,
    oauth: String,
}

#[derive(Debug, Deserialize)]
struct AutoJoin {
    #[serde(flatten)]
    servers: HashMap<String, Server>,
}

#[derive(Debug, Deserialize)]
struct Server {
    ip: String,
    nick: String,
    channels: Vec<String>,
}

fn read_file() -> Result<Config, Box<dyn std::error::Error>> {
    let mut path = dirs_next::home_dir().expect("could not find home dir");
    path.push(".config/rustychat/config.toml");
    if Path::new(&path).exists() {
        // File exists, you can read it
        let toml_str = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&toml_str)?;
        Ok(config)
    } else {
        //app.chat_bounds("Config File not Found".to_string(), "System".to_string(), "Status".to_string(), "Error".to_string());
        Err("Error".into())
    }
}

pub fn read_config(app: &mut App) {
    let config_result = read_file();
    
    match config_result {
        Ok(config) => {
            if let Some(conf) = &config.config {
                app.active_nick = conf.nick.clone();
                if let Some(server) = app.server_list.get_mut(&"System".to_string()) {
                    server.nick = conf.nick.clone();
                }
            } else {
                //No Config Section Found
            }
        }
        Err(_e) => {}
    }
    
}

pub fn read_twitch() -> (String, String) {
    let config_result = read_file();
    
    match config_result {
        Ok(twitch) => {
            if let Some(tw) = &twitch.twitch {
                return (tw.nick.clone(), tw.oauth.clone());
            } else {
                //No Config Section Found
                return("Twitch Entry not found".to_string(), "Error".to_string())
            
            }
        }
        Err(_e) => {("Twitch Entry not found".to_string(), "Error".to_string())}
    }
}

pub async fn read_autojoin (app: &mut App, net_tx: &tokio::sync::mpsc::UnboundedSender<(ServerId, NetEvent)>) {
    let config_result = read_file();
    
    match config_result {
        Ok(autojoin) => {
            if let Some(auto) = &autojoin.autojoin {
                for (_, server) in &auto.servers {
                    //app.chat_bounds(server.ip.to_string(), "System".to_string(), "Status".to_string(), "Error".to_string());
                    let port = ":6667".to_string();
                    let server_id: String = server.ip.to_string();
                    let addr: String = server_id.clone() + &port;

                    if app.stream_mgr.connect(server_id.clone(), addr.to_string(), net_tx.clone(), server.nick.clone(), app.real.clone(), "".to_string()).await {
                        match app.server_list.entry(server_id.clone()) {
                            Entry::Occupied(o) => o.into_mut(),
                            Entry::Vacant(v) => {
                            // Create a new HashMap with the "Status" channel already inserted
                                let mut channels = BTreeMap::new();
                                channels.insert("Status".to_string(), ChannelData::default());
                                v.insert(ServerData {
                                    channels,
                                    nick: server.nick.clone(),
                                })
                            }
                        };
                        app.active_server = server_id.clone();
                        app.active_channel = "Status".to_string();
                        app.active_nick = server.nick.clone();
                        if let Some(server) =  app.server_list.get_mut(&server_id) {
                            if let Some(channel) = server.channels.get_mut(&"Status".to_string()) {
                                channel.chat_list.push(("System".to_string(),String::from(format!("<connecting to {}>", addr))));
                            }
                        }
                    }
                }
            } else {
                //No Config Section Found
            }
        }
        Err(_e) => {}
    }
} 

pub fn autojoin_channel (app: &mut App, server_id: ServerId) {
    let config_result = read_file();
    
    match config_result {
        Ok(autojoin) => {
            if let Some(auto) = &autojoin.autojoin {
                for (_, server) in &auto.servers {
                    if server.ip == server_id {
                        for channel in server.channels.clone() {
                            app.stream_mgr.send_line(server_id.clone(), "JOIN ".to_string() + &channel);
                        }
                    }
                }
            }
        }
        Err(_e) => {}
    }
}

pub fn read_theme (app: &mut App) {
    let config_result = read_file();
    
    match config_result {
        Ok(theme) => {
            if let Some(colors) = &theme.theme {
                let [fgr, fgg, fgb]: [u8; 3] = colors.fg.as_slice().try_into().unwrap_or([255,238,140]);
                let [bgr, bgg, bgb]: [u8; 3] = colors.bg.as_slice().try_into().unwrap_or([47,50,54]);
                let [notr, notg, notb]: [u8; 3] = colors.notification.as_slice().try_into().unwrap_or([140, 255, 238]);
                let [highr, highg, highb]: [u8; 3] = colors.highlight.as_slice().try_into().unwrap_or([238, 140, 255]);
                let [txtr, txtg, txtb]: [u8; 3] = colors.text.as_slice().try_into().unwrap_or([255, 255, 255]);

                app.style_fg = (fgr, fgg, fgb);
                app.style_bg = (bgr, bgg, bgb);
                app.style_notif = (notr, notg, notb);
                app.style_highlight = (highr, highg, highb);
                app.style_txt = (txtr, txtg, txtb);
            }
        }
        Err(_e) => {}
    }
}
