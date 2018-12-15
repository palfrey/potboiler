#![allow(clippy::inconsistent_digit_grouping)]

use postgres::{self, transaction::Transaction};
use schemamama::{self, migration, Migrator};
use schemamama_postgres::{PostgresAdapter, PostgresMigration};

struct CreateLog;
migration!(CreateLog, 2016_05_22_1254, "create log table");

impl PostgresMigration for CreateLog {
    fn up(&self, transaction: &Transaction) -> Result<(), postgres::error::Error> {
        transaction
            .execute(
                "CREATE TABLE log (id UUID PRIMARY KEY, owner UUID NOT NULL, \
                 next UUID, prev UUID, data JSONB NOT NULL);",
                &[],
            )
            .unwrap();
        Ok(())
    }

    fn down(&self, transaction: &Transaction) -> Result<(), postgres::error::Error> {
        transaction.execute("DROP TABLE log;", &[]).unwrap();
        Ok(())
    }
}

struct Notifications;
migration!(Notifications, 2016_09_18_1322, "add apps to notify");

impl PostgresMigration for Notifications {
    fn up(&self, transaction: &Transaction) -> Result<(), postgres::error::Error> {
        transaction
            .execute("CREATE TABLE notifications (url VARCHAR(2083) PRIMARY KEY);", &[])
            .unwrap();
        Ok(())
    }

    fn down(&self, transaction: &Transaction) -> Result<(), postgres::error::Error> {
        transaction.execute("DROP TABLE notifications;", &[]).unwrap();
        Ok(())
    }
}

struct Timestamp;
migration!(Timestamp, 2016_10_02_2024, "add hlc timestamp to log");

impl PostgresMigration for Timestamp {
    fn up(&self, transaction: &Transaction) -> Result<(), postgres::error::Error> {
        transaction
            .execute("ALTER TABLE log ADD COLUMN hlc_tstamp BYTEA", &[])
            .unwrap();
        Ok(())
    }

    fn down(&self, transaction: &Transaction) -> Result<(), postgres::error::Error> {
        transaction
            .execute("ALTER TABLE log DROP COLUMN hlc_tstamp", &[])
            .unwrap();
        Ok(())
    }
}

struct Nodes;
migration!(Nodes, 2016_10_22_1748, "add other node listing");

impl PostgresMigration for Nodes {
    fn up(&self, transaction: &Transaction) -> Result<(), postgres::error::Error> {
        transaction
            .execute("CREATE TABLE nodes (url VARCHAR(2083) PRIMARY KEY);", &[])
            .unwrap();
        Ok(())
    }

    fn down(&self, transaction: &Transaction) -> Result<(), postgres::error::Error> {
        transaction.execute("DROP TABLE nodes", &[]).unwrap();
        Ok(())
    }
}

fn migrate(connection: &postgres::Connection) -> Migrator<PostgresAdapter> {
    let adapter = PostgresAdapter::new(connection);
    adapter.setup_schema().unwrap();

    let mut migrator = Migrator::new(adapter);
    migrator.register(Box::new(CreateLog));
    migrator.register(Box::new(Notifications));
    migrator.register(Box::new(Timestamp));
    migrator.register(Box::new(Nodes));
    migrator
}

pub fn up(connection: &postgres::Connection) -> Result<(), schemamama::Error<postgres::error::Error>> {
    let migrator = migrate(connection);
    migrator.up(None)
}
