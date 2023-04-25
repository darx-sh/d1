```bash
# JS runtime with db query support
cargo test test_db_query
```


```bash
# Please install httpie first.
# Create a JS module
http POST localhost:3000/c/modules < create_module.json
```

```bash
# invoke function with GET
http GET localhost:3000/d/api/f/foo
```