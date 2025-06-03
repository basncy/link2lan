#!/bin/bash
./udpdeminer-x86_64-unknown-linux-musl -l 12740 -s 1.2.3.4 -p 5678 --outbound 0.0.0.0 --hookpath ./ntfyhook.sh --hookip customize