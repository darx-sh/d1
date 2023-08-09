schema "darx_control" {
  charset = "utf8mb4"
  collate = "utf8mb4_unicode_ci"
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
    default = sql("CURRENT_TIMESTAMP(3)")
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
    type    = bigint
    default = 0
  }
  primary_key {
    columns = [column.id]
  }
  index "envs_project_id_idx" {
    unique = true
    columns = [column.project_id]
  }
}

# use a separate table for dbs since some env may not have any dbs,
# for example a plugin.
table "env_dbs" {
  schema  = schema.darx_control
  collate = "utf8mb4_unicode_ci"
  column "id" {
    null           = false
    type           = bigint
    auto_increment = true
  }
  column "env_id" {
    null = false
    type = varchar(191)
  }
  column "db_type" {
    null = false
    type = varchar(191)
  }
  column "db_host" {
    null = false
    type = varchar(191)
  }
  column "db_port" {
    null = false
    type = int
  }
  # todo: encrypt this
  column "db_user" {
    null = false
    type = varchar(191)
  }
  # todo: encrypt this
  column "db_password" {
    null = false
    type = varchar(191)
  }
  column "db_name" {
    null = false
    type = varchar(191)
  }
  primary_key {
    columns = [column.id]
  }
  index "env_dbs_db_user_idx" {
    unique  = true
    columns = [column.db_user]
  }
  index "env_dbs_db_name_idx" {
    unique  = true
    columns = [column.db_name]
  }
  index "env_dbs_env_id_idx" {
    unique  = true
    columns = [column.env_id]
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
  column "deploy_seq" {
    null    = false
    type    = bigint
    default = 0
  }
  primary_key {
    columns = [column.id]
  }
  index "deploys_env_id_deploy_seq_key" {
    unique  = true
    columns = [column.env_id, column.deploy_seq]
  }
}

table "codes" {
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
  column "deploy_id" {
    null = false
    type = varchar(191)
  }
  column "fs_path" {
    null = false
    type = text
  }
  column "content" {
    null = false
    # 16 MB
    type = mediumblob
  }
  column "content_size" {
    null = false
    type = int
  }
  primary_key {
    columns = [column.id]
  }
  index "codes_deployment_id_idx" {
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
  column "func_sig_version" {
    null = false
    default = 1
    type = int
  }
  column "func_sig" {
    null = false
    type = json
  }
  primary_key {
    columns = [column.id]
  }
  index "http_routes_deploy_id_idx" {
    columns = [column.deploy_id]
  }
}

table "plugins" {
  schema = schema.darx_control
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
    default = sql("CURRENT_TIMESTAMP(3)")
  }
  column "name" {
    null = false
    type = varchar(191)
  }
  column "env_id" {
    null = false
    type = varchar(191)
  }
  primary_key {
    columns = [column.id]
  }
  index "plugins_name" {
    unique = true
    columns = [column.name]
  }
  index "plugins_env_id" {
    unique = true
    columns = [column.env_id]
  }
}


table "plugin_installs" {
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
    default = sql("CURRENT_TIMESTAMP(3)")
  }
  column "env_id" {
    null = false
    type = varchar(191)
  }
  column "plugin_id" {
    null = false
    type = varchar(191)
  }
  primary_key {
    columns = [column.id]
  }
  index "plugin_installs_env_id_plugin_id_idx" {
    unique = true
    columns = [column.env_id, column.plugin_id]
  }
}

table "env_vars" {
  schema  = schema.darx_control
  collate = "utf8mb4_unicode_ci"

  column "id" {
    null = false
    type = bigint
    auto_increment = true
  }

  column "env_id" {
    null = false
    type = varchar(191)
  }

  column "key" {
    null = false
    type = varchar(191)
  }

  column "value" {
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
    default = sql("CURRENT_TIMESTAMP(3)")
    on_update = sql("CURRENT_TIMESTAMP(3)")
  }

  column "is_delete" {
    null = false
    type = bool
    default = false
  }

  primary_key  {
      columns = [column.id]
  }

  index "uk_env_key" {
    unique = true
    columns = [column.env_id, column.key, column.is_delete]
  }
}

table "deploy_vars" {
  schema  = schema.darx_control
  collate = "utf8mb4_unicode_ci"
  column "id" {
    null = false
    type = bigint
    auto_increment = true
  }
  column "deploy_id" {
    null = false
    type = varchar(191)
  }
  column "key" {
    null = false
    type = varchar(191)
  }
  column "value" {
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
    default = sql("CURRENT_TIMESTAMP(3)")
    on_update = sql("CURRENT_TIMESTAMP(3)")
  }
  primary_key {
    columns = [column.id]
  }
  index "uk_deploy_key" {
    unique = true
    columns = [column.deploy_id, column.key]
  }
}
