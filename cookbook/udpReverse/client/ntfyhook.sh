#!/bin/bash

#NATTYPE=3
NATTYPE=1

EVENT=$1
STREAMID=$2
SRVSTR=$3
LSTR=$4
TOPICURL="ntfy.sh/link2lantest"

case $EVENT in
	getsrvstr)
		./link2lan-x86_64-unknown-linux-musl --plan 101 --topicurl $TOPICURL --mynattype $NATTYPE --event $EVENT --streamid $STREAMID --localstr "${LSTR}"
		;;
        starterr)
                if [ $3 -gt 10 ];then
                        systemctl restart myscript@0-client-start
                        exit 0
                fi
                ;;
	*)
		exit 0
		;;
esac
