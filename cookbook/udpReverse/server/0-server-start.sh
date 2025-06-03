#!/bin/bash
#curl https://yourdomain.com/path/to/this.sh --location -s | bash

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

pkill ntfy

#Download binary
if [ ! -d link2lan ]; then
	VERSION=v0.1.2
	curl "https://github.com/basncy/link2lan/releases/download/${VERSION}/link2lan-x86_64-unknown-linux-musl" --location -s -o link2lan
    chmod +x link2lan
fi
if [ ! -d udpdeminer ]; then
	VERSION=v1.3.1
	curl "https://github.com/basncy/udpdeminer-binary/releases/download/${VERSION}/udpdeminer-x86_64-unknown-linux-musl" --location --retry-connrefused --retry 10 --fail -s -o udpdeminer
    chmod +x udpdeminer
fi

if [ ! -d ntfy ]; then
	VERSION=2.11.0
    curl https://github.com/binwiederhier/ntfy/releases/download/v${VERSION}/ntfy_${VERSION}_linux_amd64.tar.gz --location --retry-connrefused --retry 10 --fail -s -o ntfy_${VERSION}_linux_amd64.tar.gz
    tar zxvf ntfy_${VERSION}_linux_amd64.tar.gz
    mv ntfy_${VERSION}_linux_amd64/ntfy ntfy
fi

export TOPICURL="${NTFYHOST}/${NTFYTOPIC}"

function ntfy_worker() {
	REVENT=$1
	RSTREAMID=$2
	RSRVSTR=$3
	RLSTR=$4
	RNATTYPE=$5

	if [ "$REVENT" = "getsrvstr" ];then
		LPORT=$((RANDOM + 1025))
		if [ $RNATTYPE -eq 1 ];then
			MYNATTYPE=4
		else
			MYNATTYPE=3
		fi
		./link2lan --plan 201 --topicurl ${TOPICURL} --mynattype $MYNATTYPE --streamid $RSTREAMID --srvstr $RLSTR --localstr 0.0.0.0:${LPORT}
		pkill udpdeminer
		./udpdeminer -b 0.0.0.0 -l $LPORT --server $DESTIP --port $DESTPORT --idlehop 600 --forcehop 3600 &
		#socat is alternative to udpdeminer
		#socat "udp-listen:$LPORT" 'udp:$DESTIP:$DESTPORT' &
	fi
}

./ntfy subscribe --config ./link2lan.yml --from-config | \
        while IFS= read -r LINE0
        do
                ntfy_worker $LINE0
        done
