```bash
# JS runtime with db query support
cargo test test_db_query
```


```bash
# Example of expose REST api for JS function

# 1. Run the server
cargo run -- server

# 2. Install httpie.


# 3. Run the dev server
```bash
mkdir -p ./tmp
cargo run -- dev --dir=./tmp
```

# 4. Write your function in ./tmp/darx_server/functions/foo.js


# 5. Invoke function with POST
http localhost:4001/foo name=abc age=18 Host:cljb3ovlt0002e38vwo0xi5ge.darx.sh
```