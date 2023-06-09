#!/usr/bin/env bash
set -e

if [ "$#" -ne 1 ]; then
	echo "Please provide the number of initial validators!"
	exit 1
fi

# Copy paste your mnemonic here.
SECRET="cluster truck panther color mutual coast enhance uniform twelve sibling donor trust"

generate_account_id() {
	subkey inspect ${3:-} ${4:-} "$SECRET//$1//$2" | grep "Account ID" | awk '{ print $3 }'
}

generate_address() {
	subkey inspect ${3:-} ${4:-} "$SECRET//$1//$2" | grep "SS58 Address" | awk '{ print $3 }'
}

generate_public_key() {
	subkey inspect ${3:-} ${4:-} "$SECRET//$1//$2" | grep "Public" | awk '{ print $4 }'
}

generate_private_key() {
	subkey inspect ${3:-} ${4:-} "$SECRET//$1//$2" | grep "Secret seed" | awk '{ print $3 }'
}

generate_secret_phrase() {
	subkey inspect ${3:-} ${4:-} "$SECRET//$1//$2" | grep "Secret Key URI" | awk '{ print $0 }'
}

generate_address_and_secret_phrase() {
	ADDRESS=$(generate_address $1 $2 $3)
	PHRASE=$(generate_secret_phrase $1 $2 $3)

	printf "// $ADDRESS\n $PHRASE"
}


generate_address_and_private_key() {
	ADDRESS=$(generate_address $1 $2 $3)
	PRIVATE_KEY=$(generate_private_key $1 $2 $3)

	printf "// $ADDRESS\n $PRIVATE_KEY"
}

generate_address_and_public_key() {
	ADDRESS=$(generate_address $1 $2 $3)
	PUBLIC_KEY=$(generate_public_key $1 $2 $3)

	printf "// $ADDRESS\narray_bytes::hex2array_unchecked(\"${PUBLIC_KEY#'0x'}\").unchecked_into(),"
}

generate_address_and_account_id() {
	ACCOUNT=$(generate_account_id $1 $2 $3)
	ADDRESS=$(generate_address $1 $2 $3)
	if ${4:-false}; then
		INTO=".unchecked_into()"
		BYTES="array_bytes::hex2array_unchecked"
	else
		BYTES="array_bytes::hex_n_into_unchecked"
	fi

	printf "// $ADDRESS\n${BYTES}(\"${ACCOUNT#'0x'}\")$INTO,"
}

V_NUM=$1

AUTHORITIES=""

for i in $(seq 1 $V_NUM); do
	#AUTHORITIES+="$(generate_address_and_secret_phrase $i stash)\n"
	AUTHORITIES+="$(generate_address_and_private_key $i stash)\n"
	AUTHORITIES+="\n"
	AUTHORITIES+="(\n"
	AUTHORITIES+="$(generate_address_and_account_id $i stash)\n"
	AUTHORITIES+="$(generate_address_and_account_id $i controller)\n"
	AUTHORITIES+="$(generate_address_and_account_id $i grandpa '--scheme ed25519' true)\n"
	AUTHORITIES+="$(generate_address_and_account_id $i babe '--scheme sr25519' true)\n"
	AUTHORITIES+="$(generate_address_and_account_id $i im_online '--scheme sr25519' true)\n"
	AUTHORITIES+="$(generate_address_and_account_id $i authority_discovery '--scheme sr25519' true)\n"
	AUTHORITIES+="),\n"
done

printf "$AUTHORITIES"
#printf "$SECRET"
