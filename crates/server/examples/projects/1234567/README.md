```bash
# generate bundle
cd functions
esbuild --bundle foo_empty.js foo.js foo/foo.js --outdir=__output --platform=browser --format=esm --target=esnext --metafile=__output/meta.json
```