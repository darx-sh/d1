
# Example of expose REST api for JS function

# 1. Setup basic directory, run db migration
```bash
mkdir -p ./tmp

cd crates/server

atlas schema apply -u "mysql://root:12345678@localhost:3306/darx_control" --to file://schema.hcl
```
# 2. Start darx_server.
```bash
cargo run --package darx_server -- --data-plane-dir=./tmp/data_plane
```

# 3. Start darx dev command
```bash
cargo run --package darx_client -- dev --dir=./tmp
```

# 4. Write your function in ./tmp/darx_server/functions/foo.js

# 5. Invoke function with POST
```bash
http localhost:4001/foo name=abc age=18 Host:cljb3ovlt0002e38vwo0xi5ge.darx.sh
```