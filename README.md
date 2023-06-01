```bash
# JS runtime with db query support
cargo test test_db_query
```


```bash
# Example of expose REST api for JS function

# 1. Run the server
cargo run -- server

# 2. Install httpie.


# 3. Create a JS module
http POST localhost:4001/c/draft/modules < module_simple.json

# 4. Invoke function with POST
 echo -n '{}' | http POST localhost:4001/invoke/preview/pub/foo
 
```