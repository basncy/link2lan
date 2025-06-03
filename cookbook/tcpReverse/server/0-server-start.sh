#!/bin/bash

#Envirounment
[ "v$NTFYHOST" = "v" ] && export NTFYHOST="ntfy.sh"
[ "v$NTFYTOPIC" = "v" ] && export NTFYTOPIC="link2lantest"
[ "v$DESTIP" = "v" ] && export DESTIP=8.8.9.9
[ "v$DESTPORT" = "v" ] && export DESTPORT=8899

echo "default-host: https://$NTFYHOST
subscribe:
  - topic: $NTFYTOPIC
    command: 'echo \"\$message\"'
">link2lan.yml

pkill udpdeminer
pkill ntfy

#Download binary
if [ ! -d link2lan ]; then
	VERSION=v0.1.2
	curl "https://github.com/basncy/link2lan/releases/download/${VERSION}/link2lan-x86_64-unknown-linux-musl" --location -s -o link2lan
    chmod +x link2lan
fi
if [ ! -d udpdeminer ]; then
	VERSION=v1.3.1
	curl "https://github.com/basncy/udpdeminer-binary/releases/download/${VERSION}/udpdeminer-x86_64-unknown-linux-musl" --location -s -o udpdeminer
    chmod +x udpdeminer
fi

if [ ! -d ntfy ]; then
	VERSION=2.11.0
    curl https://github.com/binwiederhier/ntfy/releases/download/v${VERSION}/ntfy_${VERSION}_linux_amd64.tar.gz --location -s -o ntfy_${VERSION}_linux_amd64.tar.gz
    tar zxvf ntfy_${VERSION}_linux_amd64.tar.gz
    mv ntfy_${VERSION}_linux_amd64/ntfy ntfy
fi

if [ ! -d wstunnel ]; then
	VERSION=10.3.0
	curl https://github.com/erebe/wstunnel/releases/download/v${VERSION}/wstunnel_${VERSION}_linux_amd64.tar.gz --location -s -o wstunnel_${VERSION}_linux_amd64.tar.gz
    tar zxvf wstunnel_${VERSION}_linux_amd64.tar.gz
	chmod +x wstunnel
fi

export TOPICURL="${NTFYHOST}/${NTFYTOPIC}"

function ntfy_worker() {
	REVENT=$1
	RSTREAMID=$2
	RSRVSTR=$3
	RLSTR=$4
	RNATTYPE=$5
	RLPORT=$(echo $RLSTR|awk -F':' '{print $(NF)}')
	STREAM_ENV="/dev/shm/ntfy-${RSTREAMID}.env"
	case $REVENT in
		getsrvstr)
			LPORT=$((RANDOM + 1025))
			LWSPORT=$((RANDOM + 1025))
			#ws server addr stored in rsrvstr
			RWSSTR=$RSRVSTR
			./wstunnel client --log-lvl OFF -L "udp://${LWSPORT}:127.0.0.1:${RLPORT}?timeout_sec=0" "ws://${RWSSTR}" --http-headers "Host: qq.com" & echo WSPID=$! > $STREAM_ENV
			sleep 0.3
			./link2lan --plan 201 --topicurl ${TOPICURL} --mynattype 4 --streamid $RSTREAMID --srvstr 127.0.0.1:$LWSPORT --localstr 0.0.0.0:${LPORT}
			./udpdeminer -b 0.0.0.0 -l $LPORT --server $DESTIP --port $DESTPORT --idlehop 600 --forcehop 3600 --outbound 0.0.0.0 & echo UDMPID=$! >> $STREAM_ENV
			#socat "udp-listen:$LPORT" "udp:$DESTIP:$DESTPORT" & FWDPID=$!
			;;
		stoppost|stopwsc)
			source $STREAM_ENV
			kill -9 $WSPID $UDMPID
			rm $STREAM_ENV
			;;
		*)
			;;
	esac

}

./ntfy subscribe --config ./link2lan.yml --from-config | \
        while IFS= read -r LINE0
        do
                ntfy_worker $LINE0
        done