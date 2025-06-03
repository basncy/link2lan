#!/bin/bash
test1=$((RANDOM + 1025))
test2=$((test1 + 1))

./link2lan-x86_64-unknown-linux-musl --plan 1 --topicurl placeholder --localstr 0.0.0.0:${test1} --stunstr 74.125.250.129:19302
echo ""
./link2lan-x86_64-unknown-linux-musl --plan 1 --topicurl placeholder --localstr 0.0.0.0:${test1} --stunstr 162.159.207.0:3478
echo ""
./link2lan-x86_64-unknown-linux-musl --plan 1 --topicurl placeholder --localstr 0.0.0.0:${test2} --stunstr 74.125.250.129:19302
echo ""
./link2lan-x86_64-unknown-linux-musl --plan 1 --topicurl placeholder --localstr 0.0.0.0:${test2} --stunstr 162.159.207.0:3478
echo ""
