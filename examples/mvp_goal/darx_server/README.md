```bash

# darx dev should run the following commands automatically 
atlas migrate diff --dev-url sqlite://darx_dev.db --to file://schema.hcl
atlas migrate apply --url sqlite://darx.db

# darx schema deploy should run the following commands
atlas migrate apply --url sqlite://darx.db
```