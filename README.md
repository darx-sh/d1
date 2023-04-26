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
http POST localhost:3000/c/draft/modules < module_simple.json

# 4. Invoke function with GET
http GET localhost:3000/c/preview/f/foo
```