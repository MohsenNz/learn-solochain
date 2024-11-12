node_name=$1
secret_phrase=$2
password=$3

base_path=temp/"$node_name"

./target/release/solochain-template-node key insert \
    --base-path "$base_path"                        \
    --chain     customSpecRaw.json                  \
    --scheme    Sr25519                             \
    --suri      "$secret_phrase"                      \
    --password  "$password"                         \
    --key-type  aura

./target/release/solochain-template-node key insert \
    --base-path "$base_path"                        \
    --chain     customSpecRaw.json                  \
    --scheme    Ed25519                             \
    --suri      "$secret_phrase"                      \
    --password  "$password"                         \
    --key-type  gran
