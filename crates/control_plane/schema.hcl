schema "darx_control" {
  charset = "utf8mb4"
  collate = "utf8mb4_0900_ai_ci"
}

table "projects" {
  schema  = schema.darx_control
  collate = "utf8mb4_unicode_ci"
  column "id" {
    null = false
    type = varchar(191)
  }
  column "created_at" {
    null    = false
    type    = datetime(3)
    default = sql("CURRENT_TIMESTAMP(3)")
  }
  column "updated_at" {
    null = false
    type = datetime(3)
  }
  column "name" {
    null = false
    type = varchar(191)
  }
  column "organization_id" {
    null = false
    type = varchar(191)
  }
  primary_key {
    columns = [column.id]
  }
  index "projects_organization_id_idx" {
    columns = [column.organization_id]
  }
}

table "envs" {
  schema  = schema.darx_control
  collate = "utf8mb4_unicode_ci"
  column "id" {
    null = false
    type = varchar(191)
  }
  column "created_at" {
    null    = false
    type    = datetime(3)
    default = sql("CURRENT_TIMESTAMP(3)")
  }
  column "updated_at" {
    null = false
    type = datetime(3)
  }
  column "name" {
    null = false
    type = varchar(191)
  }
  column "project_id" {
    null = false
    type = varchar(191)
  }
  column "next_deploy_seq" {
    null    = false
    type    = int
    default = 0
  }
  primary_key {
    columns = [column.id]
  }
  index "envs_project_id_idx" {
    columns = [column.project_id]
  }
}

table "deploys" {
  schema  = schema.darx_control
  collate = "utf8mb4_unicode_ci"
  column "id" {
    null = false
    type = varchar(191)
  }
  column "created_at" {
    null    = false
    type    = datetime(3)
    default = sql("CURRENT_TIMESTAMP(3)")
  }
  column "updated_at" {
    null = false
    type = datetime(3)
  }
  column "tag" {
    null = true
    type = varchar(191)
  }
  column "description" {
    null = true
    type = varchar(191)
  }
  column "env_id" {
    null = false
    type = varchar(191)
  }
  column "bundle_repo" {
    null = false
    type = varchar(191)
  }
  column "bundle_upload_cnt" {
    null    = false
    type    = int
    default = 0
  }
  column "deploy_seq" {
    null    = false
    type    = int
    default = 0
  }
  column "bundle_cnt" {
    null = false
    type = int
  }
  primary_key {
    columns = [column.id]
  }
  index "deploys_env_id_deploy_seq_key" {
    unique  = true
    columns = [column.env_id, column.deploy_seq]
  }
}

table "bundles" {
  schema  = schema.darx_control
  collate = "utf8mb4_unicode_ci"
  column "id" {
    null = false
    type = varchar(191)
  }
  column "created_at" {
    null    = false
    type    = datetime(3)
    default = sql("CURRENT_TIMESTAMP(3)")
  }
  column "updated_at" {
    null = false
    type = datetime(3)
  }
  column "bytes" {
    null = false
    type = int
  }
  column "upload_status" {
    null    = false
    type    = varchar(191)
    default = "running"
  }
  column "deploy_id" {
    null = false
    type = varchar(191)
  }
  column "fs_path" {
    null = false
    type = text
  }
  column "code" {
    null = true
    # 16 MB
    type = mediumblob
  }
  primary_key {
    columns = [column.id]
  }
  index "bundles_deployment_id_idx" {
    columns = [column.deploy_id]
  }
}

table "http_routes" {
  schema  = schema.darx_control
  collate = "utf8mb4_unicode_ci"
  column "id" {
    null = false
    type = varchar(191)
  }
  column "created_at" {
    null    = false
    type    = datetime(3)
    default = sql("CURRENT_TIMESTAMP(3)")
  }
  column "updated_at" {
    null = false
    type = datetime(3)
  }
  column "method" {
    null = false
    type = varchar(191)
  }
  column "js_entry_point" {
    null = false
    type = varchar(191)
  }
  column "js_export" {
    null = false
    type = varchar(191)
  }
  column "deploy_id" {
    null = false
    type = varchar(191)
  }
  column "http_path" {
    null = false
    type = varchar(191)
  }
  primary_key {
    columns = [column.id]
  }
  index "http_routes_deploy_id_idx" {
    columns = [column.deploy_id]
  }
}

table "env_vars" {
  schema  = schema.darx_control
  collate = "utf8mb4_unicode_ci"
  column "id" {
    null = false
    type = varchar(191)
  }
  column "created_at" {
    null    = false
    type    = datetime(3)
    default = sql("CURRENT_TIMESTAMP(3)")
  }
  column "updated_at" {
    null = false
    type = datetime(3)
  }
  column "key" {
    null = false
    type = varchar(191)
  }
  column "value" {
    null = false
    type = varchar(191)
  }
  column "deploy_id" {
    null = false
    type = varchar(191)
  }
  primary_key {
    columns = [column.id]
  }
  index "env_vars_deploy_id_idx" {
    columns = [column.deploy_id]
  }
}