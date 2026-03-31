/*
 * Copyright (c) 2025-2026 Yang Chen <yang.chen@linuxe.org>
 *
 * This code is licensed under the terms of
 * GNU Affero General Public License v3.0
 */

use std::{fmt::Write, net::{IpAddr, SocketAddr, UdpSocket}, path::Path, process::{Command, exit}, thread::sleep, time::Duration};
use stunclient::{self, StunClient};
use serde::{Serialize,Deserialize};
use rand::{seq::IteratorRandom, distr::Alphanumeric, RngExt,Rng};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use sha2::{Sha256, Digest};

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, Key, XNonce
};

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

fn parse_range(portstr:&str) -> Result<(u16,u16), String> {
    if let Some((left, right))= portstr.split_once('-') {
        return Ok((left.parse::<u16>().unwrap(), right.parse::<u16>().unwrap()));
    } else {
        return Ok((portstr.parse::<u16>().unwrap(), portstr.parse::<u16>().unwrap()));
    }
}

pub fn gen_endpoint(pattern: &str) -> Result<String, String> {
    let mut rng = rand::rng();

    let (ip_part, port_part) = pattern
        .rsplit_once(':')
        .ok_or_else(|| "Format Error: ':' NOT found".to_string())?;

    let (pstart, pend) = parse_range(&port_part).unwrap();
    let final_port: u16 = rng.random_range(pstart..=pend);
    let generated_ip_str = if ip_part.starts_with('[') && ip_part.ends_with(']') {
        // --- IPv6 ---
        let inner = &ip_part[1..ip_part.len() - 1];
        let parts: Vec<String> = inner
            .split(':')
            .map(|s| {
                if s.eq_ignore_ascii_case("x") {
                    format!("{:x}", rng.random_range(0..=0xffff))
                } else {
                    s.to_string()
                }
            })
            .collect();
        parts.join(":")
    } else {
        // --- IPv4 ---
        let parts: Vec<String> = ip_part
            .split('.')
            .map(|s| {
                if s.eq_ignore_ascii_case("x") {
                    rng.random_range(0..=255).to_string()
                } else {
                    s.to_string()
                }
            })
            .collect();
        parts.join(".")
    };

    let final_ip: IpAddr = generated_ip_str
        .parse()
        .map_err(|e| format!("wrong IP ({}): {}", generated_ip_str, e))?;

    Ok(SocketAddr::new(final_ip, final_port).to_string())
}

pub fn udptest(localstr:&str, srvstr:&str, hex_input: &str) {
    let packet_data: Vec<u8> = if hex_input.is_empty() {
        rand::rng().sample_iter(&Alphanumeric).take(64)
            .map(|b| b as u8).collect()
    } else {
        hex_input.as_bytes().chunks(2)
            .filter_map(|chunk| {
                let s = std::str::from_utf8(chunk).ok()?;
                u8::from_str_radix(s, 16).ok()
            }).collect()
    };
    sleep(Duration::from_millis(5));
    let bindsocket = UdpSocket::bind(localstr).unwrap();
    bindsocket.send_to(&packet_data, srvstr).expect("udptest send failed");
}

pub fn stunclient(fmtjson:u8, localstr:&str, stunstr:&str) -> String {
    let mut rng = rand::rng();
    let stunsrv = stunstr.split(',').map(|s| s.trim()).filter(|s| !s.is_empty())
        .choose(&mut rng).expect("Invalid stunstr");
    let sc = StunClient::new(stunsrv.parse::<SocketAddr>().unwrap());
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

pub async fn publish_over_udp(udpsrvstr:&str, cryptkey:&str, event:&str, streamid:u64, srvstr:&str, localstr:&str, nattype:u8) {
    let target = gen_endpoint(udpsrvstr).expect("invalid udpsrvstr");
    let actual_message = format!("{} {} {} {} {}", event, streamid, srvstr, localstr, nattype);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let payload_str = format!("{}|{}", now, actual_message);

    // 4. XChaCha20-Poly1305
    let mut hasher = Sha256::new();
    hasher.update(cryptkey.as_bytes());
    let key_bytes = hasher.finalize();
    let key = Key::from_slice(&key_bytes);
    let cipher = XChaCha20Poly1305::new(key);
    let mut nonce_bytes = [0u8; 24];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, payload_str.as_bytes())
        .map_err(|_| "Failed to encrypt").unwrap();
    let mut payload = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&ciphertext);

    let mut hex_string = String::with_capacity(payload.len() * 2);
    for byte in payload {
        write!(&mut hex_string, "{:02x}", byte).unwrap();
    }

    if target.matches(':').count() >= 2 {
        udptest("[::]:0", &target, &hex_string);
    } else {
        udptest("0.0.0.0:0", &target, &hex_string);
    }
    sleep(Duration::from_millis(66));
}

pub async fn udp_to_proc(localstr:&str, cryptkey:&str, cmdpath:&str) {
    if cryptkey.starts_with("key") {
        println!("ERROR: Refuse to start with default cryptkey, please run with --cryptkey or L2L_CRYPTKEY (see more with '--help')");
        sleep(Duration::from_secs(1));
        exit(1);
    }
    let mut hasher = Sha256::new();
    hasher.update(cryptkey.as_bytes());
    let key_bytes = hasher.finalize();
    let key = Key::from_slice(&key_bytes);
    let cipher = Arc::new(XChaCha20Poly1305::new(key));

    let procpath = Arc::new(cmdpath.to_string());

    let socket = UdpSocket::bind(localstr).unwrap();
    println!("listen on udp://{}", socket.local_addr().unwrap().to_string());
    let mut buf = vec![0u8; 1500];

    loop {
        let (len, peer_addr) = socket.recv_from(&mut buf).unwrap();
        let payload = buf[..len].to_vec();
        let cipher_clone = Arc::clone(&cipher);
        let procpath_clone = Arc::clone(&procpath);

        tokio::spawn(async move {

            if payload.len() < 40 { return; }

            let (nonce_bytes, ciphertext) = payload.split_at(24);
            let nonce = XNonce::from_slice(nonce_bytes);

            if let Ok(plaintext_bytes) = cipher_clone.decrypt(nonce, ciphertext) {
                if let Ok(msg_str) = String::from_utf8(plaintext_bytes) {
                    if let Some((ts_str, actual_msg)) = msg_str.split_once('|') {
                        if let Ok(packet_ts) = ts_str.parse::<u64>() {
                            let current_ts = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs();

                            if current_ts.abs_diff(packet_ts) < 10 {
                                println!("Got request from {}, call {}", peer_addr, procpath_clone);

                                let _ = Command::new(&*procpath_clone)
                                    .args(actual_msg.split_whitespace())
                                    .spawn().expect("call script error").wait();
                            }
                        }
                    }
                }
            } else {
                eprintln!("Decrypt Failed, drop data from {}", peer_addr);
            }
        });
    }
}
