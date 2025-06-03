#!/bin/bash

#Envirounment
[ "v$NTFYHOST" = "v" ] && export NTFYHOST="ntfy.sh"
[ "v$NTFYTOPIC" = "v" ] && export NTFYTOPIC="link2lantest"
[ "v$DESTIP" = "v" ] && export DESTIP=8.8.9.9
[ "v$DESTPORT" = "v" ] && export DESTPORT=8899
[ "v$N4IP" = "v" ] && export N4IP=9.9.8.8

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

export CURNFWD=0
export TOPICURL="${NTFYHOST}/${NTFYTOPIC}"

function ntfy_worker() {
	REVENT=$1
	RSTREAMID=$2
	RSRVSTR=$3
	RLSTR=$4
	RNATTYPE=$5
	RLPORT=$(echo $RLSTR|awk -F':' '{print $(NF)}')
	STREAM_ENV="/dev/shm/ntfy-n4s.env"
	case $REVENT in
		getsrvstr)
			source $STREAM_ENV
			kill -9 $N4SPID
			LPORT=$((RANDOM + 1025))
			python ./n4.py -s -l $LPORT & echo N4SPID=$! > $STREAM_ENV
			curl -s -d "respsrvstr $RSTREAMID $RLSTR $N4IP:$LPORT" https://$TOPICURL
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
