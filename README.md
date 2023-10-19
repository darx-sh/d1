This is our first attempt to build a BaaS. It uses a customized Javascript runtime built for database.

This project is not maintained anymore.

We are working on a new version of Darx.

# Play with Dashboard

```bash
# 1. install pnpm and install dependencies
npm i -g pnpm
pnpm install

# 2. prepare environment variable files
cp .env.dashboard.example .env.dashboard
cp .env.server.example .env.server

# 3. prepare database
# Change DATABASE_URL in .env.server to use your own database
# and execute database migrations
atlas schema apply -u "mysql://root:12345678@localhost:3306/darx_control" --to file://crates/control_plane/schema.hcl

# 4. start server
pnpm run dev:s

# 5. install plugins
cargo run -p darx_client deploy -p schema -d ./plugins/schema
cargo run -p darx_client deploy -p table -d ./plugins/table

# 5. start dashboard
pnpm run dev:d

# 6. build server
pnpm build:s
```
