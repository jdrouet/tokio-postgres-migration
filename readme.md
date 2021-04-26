# Tokio Postgres migration

Simple library to run postgres migrations

```rust
use tokio_postgres_migration::Migration;

const SCRIPTS_UP: [(&str, &str); 2] = [
    (
        "0001-create-table-users",
        include_str!("../assets/0001-create-table-users-up.sql"),
    ),
    (
        "0002-create-table-pets",
        include_str!("../assets/0002-create-table-pets-up.sql"),
    ),
];

const SCRIPTS_DOWN: [(&str, &str); 2] = [
    (
        "0002-create-table-pets",
        include_str!("../assets/0002-create-table-pets-down.sql"),
    ),
    (
        "0001-create-table-users",
        include_str!("../assets/0001-create-table-users-down.sql"),
    ),
];

let mut client = build_postgres_client().await?;
let migration = Migration::new("table_to_keep_migrations".to_string());
// execute non existing migrations
migration.up(&mut client, &SCRIPTS_UP).await?;
// execute existing migrations
migration.down(&mut client, &SCRIPTS_DOWN).await?;
```

