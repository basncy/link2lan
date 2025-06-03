#!/bin/bash
#curl https://yourdomian.com/path/to/this.sh --location -s | bash

#Envirounment
[ "v$NTFYHOST" = "v" ] && export NTFYHOST="ntfy.sh"
[ "v$NTFYTOPIC" = "v" ] && export NTFYTOPIC="link2lantest"
[ "v$DESTIP" = "v" ] && export DESTIP=8.8.9.9
[ "v$DESTPORT" = "v" ] && export DESTPORT=8899

echo "default-host: https://$NTFYHOST
subscribe:
  - topic: link2lantest
    command: 'echo \"\$message\"'
">link2lan.yml

pkill ntfy

#Download binary
if [ ! -d link2lan ]; then
	VERSION=v0.1.2
	curl "https://github.com/basncy/link2lan/releases/download/${VERSION}/link2lan-x86_64-unknown-linux-musl" --location -s -o link2lan
    chmod +x link2lan
fi

if [ ! -d n4.py ]; then
    curl "https://raw.githubusercontent.com/basncy/udpdeminer-binary/refs/heads/main/samples/natTraversal/n4.py" --location -s -o n4.py
    chmod +x n4.py
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

export TOPICURL="${NTFYHOST}/${NTFYTOPIC}"

function ntfy_worker() {
	REVENT=$1
	RSTREAMID=$2
	RSRVSTR=$3
	RLSTR=$4
	RNATTYPE=$5
	RLPORT=$(echo $RLSTR|awk -F':' '{print $(NF)}')
	case $REVENT in
		respsrvstr)
			LPORT=$((RANDOM + 1025))
			N4IP=$(echo $RLSTR|awk -F':' '{print $1}')
			N4PORT=$(echo $RLSTR|awk -F':' '{print $(NF)}')
			N4RES=$(timeout -k 9 20 python ./n4.py -c -h $N4IP -p $N4PORT -b $LPORT)
			RESLPORT=$(echo $N4RES|awk -F'-' '{print $(NF)}')
			pkill udpdeminer
			./udpdeminer -b 0.0.0.0 -l $RESLPORT --server $DESTIP --port $DESTPORT --idlehop 600 --forcehop 3600 --outbound 0.0.0.0 &
			#socat is alternative for udpdeminer
			#socat "udp-listen:$LPORT" "udp:$DESTIP:$DESTPORT" &
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
