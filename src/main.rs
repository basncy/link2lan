use std::{sync::mpsc, time::Duration};
use clap::Parser;
use tokio::{task::JoinError, time::{error::Elapsed, timeout}};
use ntfy::{ntfy_publish, ntfy_subscribe_event};
use tools::{crudestunclient, crudestunserver, getsrvstr_from_n4, udptest};
mod ntfy;
mod tools;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Choose a plan
    #[arg(long, default_value_t = 99)]
    plan: u8,

    /// ntfy topic url to exchange peer info.
    #[arg(long, default_value_t = String::from("ntfy.sh/link2lantest"))]
    topicurl: String,

    /// Local nat type.
    #[arg(long, default_value_t = 3)]
    mynattype: u8,

    /// event
    #[arg(long, default_value_t = String::from("getsrvstr"))]
    event: String,

    /// streamid.
    #[arg(long, default_value_t = 0)]
    streamid: u64,

    /// server ip:port
    #[arg(long, default_value_t = String::from("0.0.0.0:0"))]
    srvstr: String,

    /// local ip:port
    #[arg(long, default_value_t = String::from("0.0.0.0:0"))]
    localstr: String,

    /// STUN ip:port, Domain NOT supported.
    #[arg(long, default_value_t = String::from("162.159.207.0:3478"))]
    stunstr: String,
}

async fn wait_with_timeout(handle:tokio::task::JoinHandle<()>, waitseconds:u64) -> Result<Result<(), JoinError>, Elapsed>{
    let task_duration = Duration::from_secs(waitseconds);
    return timeout(task_duration, handle).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut mapaddr=String::new();
    let reqevent=String::from("getsrvstr");
    let respevent=String::from("respsrvstr");

    match args.plan {
        //xx function test
        1 => {
            mapaddr.push_str(&tools::stunclient(1, &args.localstr, &args.stunstr));
            print!("{}", mapaddr);
        },
        2 => {
            udptest(&args.localstr, &args.srvstr);
        },
        3 => {
            ntfy_publish(&args.topicurl, &args.event, args.streamid, &args.srvstr, &args.localstr,args.mynattype).await;
        },
        4 => {
            let handle = tokio::spawn(crudestunserver(3600,args.localstr));
            let _ = handle.await;
        },
        41 => {
            println!("{}", crudestunclient(0, "0.0.0.0:0", &args.stunstr));
            println!("{}", crudestunclient(1, "0.0.0.0:0", &args.stunstr));
        },
        //1xx for client
        101 => {
            let (tx,rx) = mpsc::channel();
            let handle=tokio::spawn(ntfy_subscribe_event(tx, args.topicurl.clone(), respevent, args.streamid));
            mapaddr.push_str(&tools::stunclient(0, &args.localstr, &args.stunstr));
            if args.mynattype == 1 {
                let _ = tokio::spawn(crudestunserver(3,args.localstr.clone()));
            }
            ntfy_publish(&args.topicurl, &reqevent, args.streamid, &args.srvstr, &mapaddr,args.mynattype).await;
            if wait_with_timeout(handle, 2).await.is_ok() {
                println!("{}",&rx.recv().unwrap());
            }
        },
        102 => {
            udptest(&args.localstr, &args.srvstr);
            ntfy_publish(&args.topicurl, &args.event, args.streamid, &args.srvstr, &args.localstr,args.mynattype).await;
        },
        103 => {
            mapaddr.push_str(&tools::stunclient(0, &args.localstr, &args.stunstr));
            udptest(&args.localstr, &args.srvstr);
            ntfy_publish(&args.topicurl, &args.event, args.streamid, &args.srvstr, &mapaddr,args.mynattype).await;
        },
        104 => {
            /* n4.py is unstable on Multiprocessing, do NOT use this plan if you have public IP */
            let (tx,rx) = mpsc::channel();
            let handle=tokio::spawn(ntfy_subscribe_event(tx, args.topicurl.clone(), respevent, args.streamid));
            ntfy_publish(&args.topicurl, &reqevent, args.streamid, &args.srvstr, &args.localstr, args.mynattype).await;
            if wait_with_timeout(handle, 2).await.is_ok() {
                let resstr=rx.recv().unwrap();
                let (n4srvstr,_)=resstr.split_once("-").unwrap();
                let (n4host,n4port)=n4srvstr.split_once(":").unwrap();
                let (_, lport)=args.localstr.split_once(":").unwrap();
                let n4handle=tokio::spawn(getsrvstr_from_n4(n4host.to_string(), n4port.to_string(),lport.to_string()));
                if wait_with_timeout(n4handle, 3).await.is_ok() {
                    return Ok(());
                };
            }
        },
        //2xx for server
        201 => {
            if args.mynattype == 4 {
                mapaddr.push_str(&crudestunclient(0,&args.localstr, &args.srvstr));
            } else {
                mapaddr.push_str(&tools::stunclient(0, &args.localstr, &args.stunstr));
                udptest(&args.localstr, &args.srvstr);
            }
            ntfy_publish(&args.topicurl, &respevent, args.streamid, &args.srvstr, &mapaddr, args.mynattype).await;
        },
        _ => {
            println!("unknow plan {}\n Rung some test.", args.plan);
            mapaddr.push_str(&tools::stunclient(1, &args.localstr, &args.stunstr));
            println!("{}", mapaddr);
        }
    }
    Ok(())
}