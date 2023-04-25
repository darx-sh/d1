```bash
# JS runtime with db query support
cargo test test_db_query
```


```bash
# Create a JS module
http POST localhost:3000/c/modules < create_module.json
```

```bash
# invoke function with POST
http POST localhost:3000/d/api/f/foo


# invoke function with GET
http GET localhost:3000/d/api/f/foo
```