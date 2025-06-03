#!/bin/bash

EVENT=$1
STREAMID=$2
SRVSTR=$3
LSTR=$4
TOPICURL="ntfy.sh/link2lantest"

case $EVENT in
	getsrvstr)
		./link2lan-x86_64-unknown-linux-musl --plan 104 --topicurl $TOPICURL --mynattype 3 --event $EVENT --streamid $STREAMID --srvstr "$SRVSTR" --localstr "$LSTR"
		;;
	*)
		exit 0
		;;
esac
