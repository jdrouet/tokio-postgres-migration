use tokio_postgres::Client;

pub struct Migration {
    tablename: String,
}

impl Migration {
    pub fn new(tablename: String) -> Self {
        Self { tablename }
    }

    async fn execute_script(
        &self,
        client: &Client,
        content: &str,
    ) -> Result<(), tokio_postgres::Error> {
        client.batch_execute(content).await?;
        Ok(())
    }

    async fn insert_migration(
        &self,
        client: &Client,
        name: &str,
    ) -> Result<(), tokio_postgres::Error> {
        let query = format!("INSERT INTO {} (name) VALUES ($1)", self.tablename);
        let stmt = client.prepare(&query).await?;
        client.execute(&stmt, &[&name]).await?;
        Ok(())
    }

    async fn delete_migration(
        &self,
        client: &Client,
        name: &str,
    ) -> Result<(), tokio_postgres::Error> {
        let query = format!("DELETE FROM {} WHERE name = $1", self.tablename);
        let stmt = client.prepare(&query).await?;
        client.execute(&stmt, &[&name]).await?;
        Ok(())
    }

    async fn create_table(
        &self,
        client: &Client,
    ) -> Result<(), tokio_postgres::Error> {
        log::debug!("creating migration table {}", self.tablename);
        let query = format!(
            r#"CREATE TABLE IF NOT EXISTS {} ( name TEXT NOT NULL PRIMARY KEY, executed_at TIMESTAMP NOT NULL DEFAULT NOW() )"#,
            self.tablename
        );
        self.execute_script(client, &query).await?;
        Ok(())
    }

    async fn exists(
        &self,
        client: &Client,
        name: &str,
    ) -> Result<bool, tokio_postgres::Error> {
        log::trace!("check if migration {} exists", name);
        let query = format!("SELECT COUNT(*) FROM {} WHERE name = $1", self.tablename);
        let stmt = client.prepare(&query).await?;
        let row = client.query_one(&stmt, &[&name]).await?;
        let count: i64 = row.get(0);

        Ok(count > 0)
    }

    /// Migrate all scripts up
    pub async fn up(
        &self,
        client: &mut Client,
        scripts: &[(&str, &str)],
    ) -> Result<(), tokio_postgres::Error> {
        log::info!("migrating up to {}", self.tablename);
        self.create_table(client).await?;
        for (name, script) in scripts {
            if !self.exists(client, name).await? {
                log::debug!("deleting migration {}", name);
                self.execute_script(client, script).await?;
                self.insert_migration(client, name).await?;
            }
        }
        Ok(())
    }

    /// Migrate all scripts down
    pub async fn down(
        &self,
        client: &Client,
        scripts: &[(&str, &str)],
    ) -> Result<(), tokio_postgres::Error> {
        log::info!("migrating down to {}", self.tablename);
        self.create_table(client).await?;
        for (name, script) in scripts {
            if self.exists(client, name).await? {
                log::debug!("deleting migration {}", name);
                self.execute_script(client, script).await?;
                self.delete_migration(client, name).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Migration;
    use std::str::FromStr;

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

    fn get_url() -> String {
        std::env::var("POSTGRES_URL").unwrap_or_else(|_| {
            "postgres://postgres@localhost:5432/postgres?connect_timeout=5".to_string()
        })
    }

    fn get_config() -> tokio_postgres::Config {
        tokio_postgres::Config::from_str(&get_url()).unwrap()
    }

    async fn get_client() -> tokio_postgres::Client {
        let cfg = get_config();
        let (client, con) = cfg.connect(tokio_postgres::NoTls).await.unwrap();
        tokio::spawn(con);
        client
    }

    #[tokio::test]
    async fn migrating() {
        let mut client = get_client().await;
        let migration = Migration::new("table_name".to_string());
        migration.up(&mut client, &SCRIPTS_UP).await.unwrap();
        migration.down(&mut client, &SCRIPTS_DOWN).await.unwrap();
    }
}
