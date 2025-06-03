# link2lan

A tool to help to connect UDP server behind LAN.

## Feature:
0. optimize for realtime, < 2s per reverse hole.
1. Zero protocol overhead, extremely fast.
2. 1-rtt, punch then exit, no daemonize.
4. No associate punching server(with ntfy.sh or selfhost)

## Network Topology

Data Stream:
```
 UDP client(call link2lan) -> Internet(original protocol) -> UDP server(ntfy subscription trigger)
```
Signal:
```
link2lan(addr from stun) -> ntfy server(deploy addr info)  <- ntfy-cli(addr from stun)
```

## !!! MUST READ !!!
The **ntfy topic** is your **private SECRETE**, we send plain text to that topic! Transportation is secured by HTTPS. If you don't trust public service, setup your own [ntfy.sh](https://docs.ntfy.sh/install/#general-steps) service, also unlock rate limit.

## Cookbook connection flow:

### Before:
```
UDP client -> >>> Internet(original) >>> -> UDP server(public IP)
```
### After

Case 1, Reverse connection:
```
UDP client(public IP) -> <<< Internet(original) <<< -> UDP server (private IP, 192.168.100.123)
```
```
UDP client(public IP) -> <<< Internet(original) <<< -> Container, Cloud Services -> UDP server(public IP)
```
Case 2, Nat Punch, experimental, unstable:
```
UDP client(192.168.0.123) -> <<< Internet(original) <<< -> UDP server(100.64.1.234)
```
```
UDP client(192.168.0.123) -> <<< Internet(original) <<< -> Container, Cloud Service -> UDP server(public IP)
```
Case 3, Reverse, over WARP, protect server:
```
UDP client(public IP) -> <<< Internet(original protocol) <<< WARP client(udp over warp) -> UDP server
```
Case 4: Your personal magic combination.
