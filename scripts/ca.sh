#!/usr/bin/env bash

OUT=$(pwd)/tests/ssl
CANAME=localCA
HOSTNAME=localhost
PASS=1234

mkdir -p $OUT

openssl genrsa -des3 -out "$OUT/$CANAME.key" -passout "pass:$PASS" 2048  \
	&& echo "✅ CA private key: '$OUT/$CANAME.key'"

openssl req \
	-x509 \
	-new \
	-nodes \
	-key "$OUT/$CANAME.key" \
	-passin "pass:$PASS" \
	-sha256 \
	-days 825 \
	-out "$OUT/$CANAME.pem" \
	-subj "/C=US/ST=Denial/L=Springfield/O=Dis/CN=www.example.com" \
		&& echo "✅ CA root certificate: '$OUT/$CANAME.pem'"

openssl genrsa -out "$OUT/$HOSTNAME.key" 2048 \
	&& echo "✅ Private key: '$OUT/$HOSTNAME.key'"

openssl req \
	-new \
	-key "$OUT/$HOSTNAME.key" \
	-out "$OUT/$HOSTNAME.csr" \
	-subj "/C=US/ST=Denial/L=Springfield/O=Dis/CN=www.example.com" \
		&& echo "✅ Certificate signing request: '$OUT/$HOSTNAME.csr'"

# Create a config file for the extensions
>"$OUT/$HOSTNAME.ext" cat <<-EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names
[alt_names]
DNS.1 = $HOSTNAME
IP.1 = 127.0.0.1
EOF

openssl x509 \
	-req \
	-in "$OUT/$HOSTNAME.csr" \
	-CA "$OUT/$CANAME.pem" \
	-CAkey "$OUT/$CANAME.key" \
	-passin "pass:$PASS" \
	-CAcreateserial \
	-out "$OUT/$HOSTNAME.crt" \
	-days 825 \
	-sha256 \
	-extfile "$OUT/$HOSTNAME.ext" \
		&& echo "✅ Signed certificate: '$OUT/$HOSTNAME.crt'"
