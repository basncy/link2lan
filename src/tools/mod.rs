/*
 * Copyright (c) 2025 Yang Chen <yang.chen@linuxe.org>
 *
 * This code is licensed under the terms of
 * GNU Affero General Public License v3.0
 */

use std::{net::{SocketAddr, UdpSocket}, path::Path, process::Command, time::Duration};
use stunclient::{self, StunClient};
use serde::{Serialize,Deserialize};
use rand::Rng;
use rand::distr::Alphanumeric;

#[derive(Serialize, Deserialize)]
pub struct StunInfo {
    pub stunstr: String,
    pub localstr: String,
    pub localip: String,
    pub localport: String,
    pub mappedstr: String,
    pub mappedip: String,
    pub mappedport: String,
}

pub fn udptest(localstr:&str, srvstr:&str) {
    let bindsocket = UdpSocket::bind(localstr).unwrap();
    let mut rng = rand::rng();
    let secret_data: String = (1..64).map(|_| rng.sample(Alphanumeric) as char).collect();
    bindsocket.send_to(secret_data.as_bytes(),srvstr).unwrap();
}

pub fn stunclient(fmtjson:u8, localstr:&str, stunstr:&str) -> String {
    let sc = StunClient::new(stunstr.parse::<SocketAddr>().unwrap());
    let u = UdpSocket::bind(localstr).unwrap();

    let mut stun_data = StunInfo {
        stunstr: String::from(stunstr),
        localstr: String::new(),
        localip: String::new(),
        localport: String::new(),
        mappedstr: String::new(),
        mappedip: String::new(),
        mappedport: String::new(),
    };

    match sc.query_external_address(&u) {
        Ok(x) => {
            if fmtjson == 0 {
                //mapped ip:port
                return x.to_string();
            } else {
                //json format
                stun_data.localip.push_str(&u.local_addr().unwrap().ip().to_string());
                stun_data.localport.push_str(&u.local_addr().unwrap().port().to_string());
                stun_data.mappedip.push_str(&x.ip().to_string());
                stun_data.mappedport.push_str(&x.port().to_string());
                if x.is_ipv6() {
                    stun_data.localstr.push_str(&format!("[{}]:{}", stun_data.localip, stun_data.localport));
                    stun_data.mappedstr.push_str(&*format!("[{}]:{}", stun_data.mappedip, stun_data.mappedport));
                } else {
                    stun_data.localstr.push_str(&format!("{}:{}", stun_data.localip, stun_data.localport));
                    stun_data.mappedstr.push_str(&*format!("{}:{}", stun_data.mappedip, stun_data.mappedport));
                }
                return serde_json::to_string(&stun_data).unwrap();
            }
        },
        Err(e) => {
            eprintln!("stunclient: {}", e);
            return String::from("");
        }
    }
}

pub fn crudestunclient(fmtjson:u8, localstr:&str, srvstr:&str) -> String {
    let mut res=String::new();
    let udpclient = UdpSocket::bind(localstr).unwrap();
    let mut recvbuf = vec![0u8; 1500];
    let mut rng = rand::rng();
    let secret_data: String = (1..64).map(|_| rng.sample(Alphanumeric) as char).collect();

    udpclient.set_read_timeout(Some(Duration::new(3,0))).expect("set timeout failed");
    udpclient.send_to(secret_data.as_bytes(),srvstr).unwrap();

    loop {
        match udpclient.recv(&mut recvbuf) {
            Err(ref e) => {
                if e.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                } else {
                    eprintln!("crudestunclient: {}", e);
                    break;
                }
            },
            Ok(recvlen) => {
                let recvstr=&String::from_utf8_lossy(&recvbuf[..recvlen]);
                if fmtjson == 0 {
                    //mapped ip:port
                    let stun_data:StunInfo = serde_json::from_str(recvstr).unwrap();
                    res.push_str(&stun_data.mappedstr);
                    break;
                } else {
                    //json/raw format
                    res.push_str(recvstr);
                    break;
                }
            }
        }
    }
    return res;
}

pub async fn crudestunserver(resfmt:u8, servetime:u64, localstr:String) {
    let u=UdpSocket::bind(localstr.clone()).unwrap();
    u.set_read_timeout(Some(Duration::new(servetime,0))).expect("set servetime failed");
    let mut sockbuf = vec![0u8; 1500];
    let mut stun_data = StunInfo {
        stunstr: localstr,
        localstr: String::new(),
        localip: String::new(),
        localport: String::new(),
        mappedstr: String::new(),
        mappedip: String::new(),
        mappedport: String::new(),
    };

    loop {
        match u.recv_from(&mut sockbuf) {
            Err(ref e) => {
                if e.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                } else {
                    eprintln!();
                    break;
                }
            },
            Ok((_,x)) => {
                //json format
                stun_data.localip.push_str(&u.local_addr().unwrap().ip().to_string());
                stun_data.localport.push_str(&u.local_addr().unwrap().port().to_string());
                stun_data.mappedip.push_str(&x.ip().to_string());
                stun_data.mappedport.push_str(&x.port().to_string());
                if x.is_ipv6() {
                    stun_data.localstr.push_str(&format!("[{}]:{}", stun_data.localip, stun_data.localport));
                    stun_data.mappedstr.push_str(&*format!("[{}]:{}", stun_data.mappedip, stun_data.mappedport));
                } else {
                    stun_data.localstr.push_str(&format!("{}:{}", stun_data.localip, stun_data.localport));
                    stun_data.mappedstr.push_str(&*format!("{}:{}", stun_data.mappedip, stun_data.mappedport));
                }
                match resfmt {
                    0 => {
                        u.send_to(format!("{}-{}",stun_data.mappedstr,stun_data.localport).as_bytes(),x).unwrap();
                    },
                    1 => {
                        u.send_to(&serde_json::to_string(&stun_data).unwrap().as_bytes(),x).unwrap();
                    },
                    2 => {
                        println!("{}", stun_data.mappedstr);
                    },
                    3 => {
                        println!("{}", serde_json::to_string(&stun_data).unwrap());
                    },
                    _ => {
                        println!("{}-{}", stun_data.mappedstr, stun_data.localport);
                    },
                }
                break;
            }
        }
    }
}

pub async fn getsrvstr_from_n4(n4host:String, n4port:String, lport:String) {
    let n4file = "n4.py";
    let n4path = Path::new(n4file);
    if ! n4path.exists() {
        println!("ERR: n4.py does NOT exist.");
    }
    let outputstr: String = String::from_utf8_lossy(&Command::new("python")
        .args(&["n4.py", "-c", "-h", &n4host, "-p", &n4port, "-b", &lport])
        .output().unwrap().stdout).into_owned();
    //print here for windows compatible
    println!("{outputstr}");
}