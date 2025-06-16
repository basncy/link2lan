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
 UDP client(schedule link2lan) -> Internet(raw protocol) -> UDP server(ntfy subscription trigger)
```
Signal:
```
link2lan(addr from stun) -> ntfy server(deploy addr info)  <- ntfy-cli(addr from stun)
```

## !!! MUST READ !!!
The **ntfy topic** is your **private SECRETE**, we send plain text to that topic! Transportation is secured by HTTPS. If you don't trust public service, setup your own [ntfy.sh](https://docs.ntfy.sh/install/#general-steps) service, also unlock rate limit.

[udpdeminer](https://github.com/basncy/udpdeminer-binary) is a scheduler to trigger Link2Lan in the [cookbook](https://github.com/basncy/link2lan/tree/main/cookbook), you can use another alternative application to use the Link2Lan output in a format of destip:dport-sport.

## Cookbook connection flow:

### Before:
```
NAT(UDP client) -> >>> Internet(raw protocol) >>> -> PubIP(UDP server)
```
### After

Case 1, Reverse connection:
```
PubIP(client) -> <<< Internet(raw protocol) <<< -> NAT(server)
```
```
PubIP(client) -> <<< Internet(raw protocol) <<< -> NAT(relay) -> PubIP(server)
```
Case 2, Nat Punch, experimental, unstable:
```
NAT3(client) -> <<< Internet(raw protocol) <<< -> NAT3(server)
```
```
NAT3(client) -> <<< Internet(raw protocol) <<< -> NAT3(relay) -> PubIP(server)
```
Case 3, Reverse, over WARP, protect server:
```
PubIP(client) -> <<< Internet(raw protocol) <<< WARP(udp over warp) -> NAT(server)
```
Case 4: Your personal magic combination.
