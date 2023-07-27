# Play with Dashboard

```
# 1. install pnpm using npm or https://pnpm.io/installation
npm i -g pnpm

# 2. prepare environment variable files
cp .env.dashboard.example .env.dashboard
cp .env.server.example .env.server

# 3. prepare database
# Change DATABASE_URL in .env.server to use your own database
# and execute database migrations 
atlas schema apply -u "mysql://root:12345678@localhost:3306/darx_control" --to file://crates/control_plane/schema.hcl

# 4. start server
pnpm run dev:s

# 5. start dashboard
pnpm run dev:d

# 6. build server
pnpm build:s
``