#!/bin/bash
[ "v$LWSIP" = "v" ] && export LWSIP=1.2.3.4
#[ "v$LWSIP" = "v" ] && export LWSIP="[2001::1]"

EVENT=$1
STREAMID=$2
SRVSTR=$3
LSTR=$4
STREAM_ENV="/dev/shm/ntfy-${STREAMID}.env"
TOPICURL=ntfy.sh/link2lantest
case $EVENT in
	getsrvstr)
		LPORT=$(echo $LSTR|awk -F':' '{print $(NF)}')
		LWSPORT=$((RANDOM + 1025))
		echo "/home/Sync/srelay/wstunnel server --log-lvl OFF --restrict-to 127.0.0.1:$LPORT ws://0.0.0.0:$LWSPORT" > "ws${STREAMID}.sh"
		chmod +x ws${STREAMID}.sh
		#MUST use third app(here is systemd) to start wstunnel as deamon, allowing this script exit without child process.
		systemctl start link2lantcp@ws${STREAMID}
		./link2lan-x86_64-unknown-linux-musl --plan 101 --topicurl $TOPICURL --mynattype 1 --event $EVENT --streamid $STREAMID --srvstr "$LWSIP:$LWSPORT" --localstr "${LSTR}"
		exit 0
		;;
	starterr)
		if [ $3 -gt 10 ];then
			systemctl restart myudpclient
			exit 0
		fi
		;;
	stoppost)
		curl -s -d "stopwsc $2" https://$TOPICURL
		systemctl stop link2lantcp@ws${STREAMID}
		rm ws${STREAMID}.sh
		exit 0
		;;
	*)
		echo "unknow event for $STREAMID"
		exit 0
		;;
esac
