CREATE TABLE IF NOT EXISTS deployments (
    id INTEGER AUTOINCREMENT,
    -- 0: Schema
    -- 1: Function
    project_id INTEGER NOT NULL,
    type INTEGER NOT NULL,
    -- 0: Doing
    -- 1: Done
    -- 2: Failed
    status INTEGER NOT NULL DEFAULT 0,
    tag TEXT,
    description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(id)
);

CREATE TABLE IF NOT EXISTS current_deployment (
    project_id INTEGER NOT NULL,
    deployment_id INTEGER NOT NULL,
    FOREIGN KEY(deployment_id) REFERENCES deployments(id),
    PRIMARY KEY(deployment_id)
);

CREATE TABLE IF NOT EXISTS db_migrations (
    file_name TEXT NOT NULL,
    sql TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 0,
    deployment_id INTEGER NOT NULL,
    --   executed_at DATETIME NULL,
    --   execution_time INTEGER NULL,
    error TEXT NULL,
    PRIMARY KEY(file_name),
    FOREIGN KEY(deployment_id) REFERENCES deployments(id)
);


CREATE TABLE IF NOT EXISTS js_bundles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL,
    code BLOB NOT NULL,
    deployment_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(deployment_id) REFERENCES deployments(id)
);

CREATE TABLE IF NOT EXISTS js_bundle_metas (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    content       BLOB     NOT NULL,
    deployment_id INTEGER  NOT NULL,
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (deployment_id) REFERENCES deployments (id)
)