/*
 * Copyright (c) 2024 Yang Chen <yang.chen@linuxe.org>
 *
 * This code is licensed under the terms of
 * GNU Affero General Public License v3.0
 */

use std::str::FromStr;
use reqwest_websocket::{Message,RequestBuilderExt};
use reqwest::Client;
use futures_util::StreamExt;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MyNtfyPayload {
    pub event: String,
}

#[derive(Deserialize)]
pub struct MyPayload {
    pub message: String,
}

pub async fn ntfy_subscribe_event(tx:std::sync::mpsc::Sender<String>, topicurl:String, regevent:String, _streamid:u64) {
    let wsconn =  Client::default()
        .get(format!("wss://{}/ws",topicurl))
        .upgrade().send().await.unwrap();

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

pub async fn ntfy_publish(topicurl:&str, event:&str, streamid:u64, srvstr:&str, localstr:&str,nattype:u8) {
    let client = reqwest::Client::new();
    //Plain text, topicurl is the secret.
    let msg_body=String::from_str(&format!("{} {} {} {} {}", event, streamid, srvstr, localstr, nattype)).unwrap();
    let _res = client.post(&format!("https://{}", topicurl))
    .body(msg_body).send().await.unwrap();
}