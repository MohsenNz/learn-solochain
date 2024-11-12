node_name=$1
password=$2
port=$3
rpc_port=$4
node_key=$5


base_path=temp/"$node_name"

./target/release/solochain-template-node                    \
    --validator                                             \
    --base-path     "$base_path"                            \
    --chain         ./customSpecRaw.json                    \
    --name          "$node_name"                            \
    --port          "$port"                                 \
    --rpc-port      "$rpc_port"                             \
    --node-key      "$node_key"                             \
    --rpc-methods   Unsafe                                  \
    --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
    --password      "$password"
