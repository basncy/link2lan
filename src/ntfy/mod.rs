/*
 * Copyright (c) 2024-2025 Yang Chen <yang.chen@linuxe.org>
 *
 * This code is licensed under the terms of
 * GNU Affero General Public License v3.0
 */

use std::{error::Error, net::{IpAddr, SocketAddr}, str::FromStr};
use reqwest::{Certificate, Client, tls::Version};
use reqwest_websocket::{Message,RequestBuilderExt};
use futures_util::{StreamExt, TryFutureExt};
use serde::Deserialize;
use tokio::fs;

#[derive(Deserialize)]
pub struct MyNtfyPayload {
    pub event: String,
}

#[derive(Deserialize)]
pub struct MyPayload {
    pub message: String,
}

fn parse_resolve_arg(resolve_str: &str) -> Result<(String, SocketAddr), Box<dyn Error>> {
    let parts: Vec<&str> = resolve_str.splitn(3, ':').collect();

    if parts.len() != 3 { return Err(format!("Resolve failed: '{}' Format: <domain>:<port>:<ip>", resolve_str).into());}

    let domain_str = parts[0];
    let port_str = parts[1];
    let mut ip_str = parts[2];

    if domain_str.is_empty() { return Err(format!("Failed to get domain from: '{}'", resolve_str).into());}
    if port_str.is_empty() { return Err(format!("Failed to get port from: '{}'", resolve_str).into());}
    if ip_str.is_empty() { return Err(format!("Failed to get ip from: '{}'", resolve_str).into());}

    if ip_str.starts_with('[') && ip_str.ends_with(']') {
        ip_str = &ip_str[1..ip_str.len() - 1];
    }

    let domain = domain_str.to_string();
    let port: u16 = port_str.parse().map_err(|e| format!("Failed to parse port '{}': {}", port_str, e))?;
    let ip: IpAddr = ip_str.parse().map_err(|e| format!("Failed to parse IP '{}': {}", ip_str, e))?;

    let addr = SocketAddr::new(ip, port);
    Ok((domain, addr))
}

async fn build_client(
    cacert_path: Option<&str>,
    resolve_arg: Option<&str>
) -> Result<Client, Box<dyn Error>> {

    let mut builder = Client::builder();

    if let Some(path) = cacert_path {
        let pem_bytes = fs::read(path).map_err(|e| format!("Failed to load CA '{}': {}", path, e)).await?;
        let cert = Certificate::from_pem(&pem_bytes).map_err(|e| format!("Failed to parse PEM: {}", e)).unwrap();
        builder = builder.add_root_certificate(cert).use_rustls_tls().min_tls_version(Version::TLS_1_3);
    }

    if let Some(resolve_str) = resolve_arg {
        let (domain, addr) = parse_resolve_arg(resolve_str)?;
        builder = builder.resolve(&domain, addr);
    }

    let client = builder.build()?;
    Ok(client)
}

pub async fn ntfy_subscribe_event(tx:std::sync::mpsc::Sender<String>, topicurl:String, cacert:Option<String>, resolve:Option<String>, regevent:String, _streamid:u64) {
    let client = build_client(cacert.as_deref(), resolve.as_deref()).await.unwrap();
    let wsconn =  client.get(format!("wss://{}/ws",topicurl)).upgrade().send().await.unwrap();

    let mut ntfyws = wsconn.into_websocket().await.unwrap();
    loop {
        let message = ntfyws.next().await.unwrap().unwrap();
        match message {
            Message::Text(data) => {
                let result:Result<MyNtfyPayload, serde_json::Error> = serde_json::from_str(&data);
                match result {
                    Ok(ntfypayload) => {
                        if ntfypayload.event.contains("message") {
                            let mypayload:MyPayload = serde_json::from_str(&data).unwrap();
                            let items: Vec<&str> = mypayload.message.split(' ').collect();
                            let revent=items[0];

                            if revent.contains(&regevent) {
                                let rstreamid=items[1];
                                let rlocalstr=items[3];
                                let _ = tx.send(format!("{rlocalstr}-{rstreamid}"));
                                break;
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            },
            _ => {
                return;
            }
        }
    }
}

pub async fn ntfy_publish(topicurl:&str, cacert:Option<&str>, resolve:Option<&str>, event:&str, streamid:u64, srvstr:&str, localstr:&str,nattype:u8) {
    let client = build_client(cacert, resolve).await.unwrap();
    let msg_body=String::from_str(&format!("{} {} {} {} {}", event, streamid, srvstr, localstr, nattype)).unwrap();
    let _res = client.post(&format!("https://{}", topicurl))
    .body(msg_body).send().await.unwrap();
}