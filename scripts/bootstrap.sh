#!/usr/bin/env bash
set -e

function usage() {
  echo "Usage: $0 -m <num_members> -n <num_nodes> -a <num_authorities> -s <SECRET> -o <output_directory>"
  echo ""
  echo "Options:"
  echo "  -m, --num-members     Number of network members"
  echo "  -n, --num-nodes       Number of authorised nodes in the network"
  echo "  -a, --num-authorities Number of authorities (validators)"
  echo "  -s, --SECRET          Secret key or seed for generating new accounts"
  echo "  -r, --runtime-type    Choose one of chain runtimes from braid, loom and weave. Default is loom."
  echo "  -o, --output-directory Directory where output files will be saved (default is current directory)"
  echo ""
  echo "Description:"
  echo "  This script generates a configuration file for the CORD Custom Chain, including"
  echo "  network members, well-known nodes, and authority nodes. You can specify the"
  echo "  number of network members, well-known nodes, authorities, and the secret key or seed"
  echo "  used for generating account information."
  echo ""
  echo "  The generated configuration is written to 'config.toml' and account details to 'accounts.txt'."
}
# Defaults
NUM_MEMBERS=6
NUM_NODES=5
NUM_AUTHORITIES=3
# Don't use accounts generated with this secret in production
SECRET="0xf32255f569d8b1a12086dfd194653a5377fafcb67345753987741ec5542920ce"
OUTPUT_DIR="."
RUNTIME_TYPE="loom"

while getopts "m:n:a:s:r:o:" flag; do
  case "${flag}" in
  m) NUM_MEMBERS=${OPTARG} ;;
  n) NUM_NODES=${OPTARG} ;;
  a) NUM_AUTHORITIES=${OPTARG} ;;
  s) SECRET=${OPTARG} ;;
  r) 
    if [[ "${OPTARG}" == "braid" || "${OPTARG}" == "weave" || "${OPTARG}" == "loom" ]]; then
      RUNTIME_TYPE=${OPTARG}
    else
      echo "Invalid runtime type. Choose one of 'braid, weave, loom'."
      exit 1
    fi
    ;;
  o) OUTPUT_DIR=${OPTARG} ;;
  *)
    usage
    exit 1
    ;;
  esac
done

shift $((OPTIND - 1))

mkdir -p "$OUTPUT_DIR"

# Initialize TOML file
CONFIG_FILE="$OUTPUT_DIR/config.toml"
echo "# Custom Chain Configuration" >$CONFIG_FILE
echo "# Auto-generated by bootstrap script" >>$CONFIG_FILE
echo "
# This configuration file defines the settings for the CORD Custom Chain.
# Please review and customize the parameters as needed for your specific use case.
" >>$CONFIG_FILE
echo "chain_name = \"CORD Custom Chain\"" >>$CONFIG_FILE
echo "chain_type = \"local\"" >>$CONFIG_FILE
echo "runtime_type = \"${RUNTIME_TYPE}\"" >>$CONFIG_FILE
echo "" >>$CONFIG_FILE

# Initialize Accounts files
ACCOUNTS_FILE="$OUTPUT_DIR/accounts.txt"
echo "# Custom Chain Configuration - Accounts & Seeds" >$ACCOUNTS_FILE
echo "
# Warning: Protect Your Account Information
# Your account information, especially the secret key, is crucial for the security of the CORD network.
# Please follow these essential security guidelines to keep your account safe and secure:

# 1. Secret Key URI: The Secret Key URI is like the master key to your account. Never share it with anyone or store it in an easily accessible location.
# 2. Secret Seed: The secret seed is the foundation of your account's security. Store it in a secure and offline location.
# 3. Public Key: While the public key (hex) and Account ID are not secret, they are associated with your account.
# 4. Public Key (SS58): This is your account's address in SS58 format. It can be safely shared for receiving funds.
# 5. SS58 Address: Your SS58 address is used for transactions and receiving funds. It is safe to share publicly.

# Remember that anyone with access to your secret key can control your account and its assets. Keep it confidential and store it securely.
# If you suspect your account's security has been compromised, take immediate action to secure it, such as transferring your assets to a new account with a new secret key.
# By following these security practices, you can help ensure the safety and integrity of your account and nodes.
" >>$ACCOUNTS_FILE
echo "" >>$ACCOUNTS_FILE

cord='./target/release/cord'

generate_account_id() {
  $cord key inspect -n cord ${2:-} ${3:-} "$SECRET//$1" | grep "Account ID" | awk '{ print $3 }'
}

generate_address() {
  $cord key inspect -n cord ${2:-} ${3:-} "$SECRET//$1" | grep "SS58 Address" | awk '{ print $3 }'
}

generate_node_key() {
    if [ ! -e $OUTPUT_DIR/node$i.key ]; then
	$cord key generate-node-key >"$OUTPUT_DIR/node$i.key" 2>/dev/null
    fi
  NODE_KEY=$($cord key inspect-node-key --file "$OUTPUT_DIR/node$i.key")
  echo "\"${NODE_KEY}\"," >>$CONFIG_FILE
}

get_account_id() {
  ACCOUNT=$(generate_account_id $1 $2)
  echo "\"${ACCOUNT#'0x'}\"," >>$CONFIG_FILE
}

get_ed_account_id() {
  ACCOUNT_SECRET=$($cord key inspect -n cord --scheme Sr25519 ${3:-} "$SECRET//$1" | grep "Secret seed" | awk '{ print $3 }')
  ACCOUNT=$($cord key inspect -n cord ${2:-} ${3:-} "$ACCOUNT_SECRET" | grep "Account ID" | awk '{ print $3 }')
  echo "\"${ACCOUNT#'0x'}\"," >>$CONFIG_FILE
}

get_ss58_address() {
  ADDRESS=$(generate_address $1 $2)
  echo "\"${ADDRESS}\"," >>$CONFIG_FILE
}

generate_account_details() {

  ACCOUNT_DETAILS=$($cord key inspect -n cord ${2:-} ${3:-} "$SECRET//$1")

  echo "Account $i" >>$ACCOUNTS_FILE
  echo "${ACCOUNT_DETAILS}" >>$ACCOUNTS_FILE
  echo "" >>$ACCOUNTS_FILE
}

echo "network_members = [" >>$CONFIG_FILE

for i in $(seq 1 $NUM_MEMBERS); do
  generate_account_details $i '--scheme Sr25519'
  echo "[" >>$CONFIG_FILE
  get_ss58_address $i '--scheme Sr25519'
  get_account_id $i '--scheme Sr25519'
  echo "  ]," >>$CONFIG_FILE
done

echo "]" >>$CONFIG_FILE
echo "" >>$CONFIG_FILE

echo "well_known_nodes = [" >>$CONFIG_FILE

for i in $(seq 1 $NUM_NODES); do
  echo "[" >>$CONFIG_FILE
  get_ss58_address $i '--scheme Sr25519'
  generate_node_key
  get_account_id $i '--scheme Sr25519'
  echo "  ]," >>$CONFIG_FILE
done

echo "]" >>$CONFIG_FILE
echo "" >>$CONFIG_FILE

echo "authorities = [" >>$CONFIG_FILE

for i in $(seq 1 $NUM_AUTHORITIES); do
  echo "[" >>$CONFIG_FILE
  get_ss58_address $i '--scheme Sr25519'
  get_account_id $i '--scheme Sr25519'
  get_ed_account_id $i '--scheme Ed25519'
  echo "]," >>$CONFIG_FILE
done

echo "]" >>$CONFIG_FILE

echo "" >>$CONFIG_FILE

echo "☃️ Chain configuration has been generated!"
