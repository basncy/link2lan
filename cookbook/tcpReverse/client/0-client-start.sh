#!/bin/bash
export LWSIP=$(dig TXT +short o-o.myaddr.l.google.com @216.239.34.10 +tcp | cut -d\" -f2)
./udpdeminer-x86_64-unknown-linux-musl -l 12740 -s 1.2.3.4 -p 5678 --outbound 0.0.0.0 --hookpath ./ntfywshook.sh --hookip customize
