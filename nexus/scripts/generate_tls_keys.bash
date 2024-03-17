#!/bin/bash
mkdir -p ./certs
cd ./certs
openssl req -new -newkey rsa:4096 -x509 -sha256 -days 365 -nodes -out ./cert.pem -keyout ./key.pem