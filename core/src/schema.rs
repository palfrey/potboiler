use postgres;
use schemamama;
use schemamama::Migrator;
use schemamama_postgres::{PostgresAdapter, PostgresMigration};

struct CreateLog;
migration!(CreateLog, 201605221254, "create log table");

impl PostgresMigration for CreateLog {
    fn up(&self, transaction: &postgres::Transaction) -> Result<(), postgres::error::Error> {
        transaction.execute(
            "CREATE TABLE log (id UUID PRIMARY KEY, owner UUID NOT NULL, \
            next UUID, prev UUID, data JSONB NOT NULL);", &[])
            .unwrap();
        return Ok(());
    }

    fn down(&self, transaction: &postgres::Transaction) -> Result<(), postgres::error::Error> {
        let _ = transaction.execute("DROP TABLE log;", &[]).unwrap();
        return Ok(());
    }
}

struct Notifications;
migration!(Notifications, 201609181322, "add apps to notify");

impl PostgresMigration for Notifications {
    fn up(&self, transaction: &postgres::Transaction) -> Result<(), postgres::error::Error> {
        transaction.execute("CREATE TABLE notifications (url VARCHAR(2083) PRIMARY KEY);",
                     &[])
            .unwrap();
        return Ok(());
    }

    fn down(&self, transaction: &postgres::Transaction) -> Result<(), postgres::error::Error> {
        let _ = transaction.execute("DROP TABLE notifications;", &[]).unwrap();
        return Ok(());
    }
}

struct Timestamp;
migration!(Timestamp, 201610022024, "add hlc timestamp to log");

impl PostgresMigration for Timestamp {
    fn up(&self, transaction: &postgres::Transaction) -> Result<(), postgres::error::Error> {
        transaction.execute("ALTER TABLE log ADD COLUMN hlc_tstamp BYTEA", &[])
            .unwrap();
        return Ok(());
    }

    fn down(&self, transaction: &postgres::Transaction) -> Result<(), postgres::error::Error> {
        let _ = transaction.execute("ALTER TABLE log DROP COLUMN hlc_tstamp", &[]).unwrap();
        return Ok(());
    }
}

struct Nodes;
migration!(Nodes, 201610221748, "add other node listing");

impl PostgresMigration for Nodes {
    fn up(&self, transaction: &postgres::Transaction) -> Result<(), postgres::error::Error> {
        transaction.execute("CREATE TABLE nodes (url VARCHAR(2083) PRIMARY KEY);", &[])
            .unwrap();
        return Ok(());
    }

    fn down(&self, transaction: &postgres::Transaction) -> Result<(), postgres::error::Error> {
        let _ = transaction.execute("DROP TABLE nodes", &[]).unwrap();
        return Ok(());
    }
}

fn migrate(connection: &postgres::Connection) -> Migrator<PostgresAdapter> {
    let adapter = PostgresAdapter::new(connection);
    let _ = adapter.setup_schema().unwrap();

    let mut migrator = Migrator::new(adapter);
    migrator.register(Box::new(CreateLog));
    migrator.register(Box::new(Notifications));
    migrator.register(Box::new(Timestamp));
    migrator.register(Box::new(Nodes));
    return migrator;
}

pub fn up(connection: &postgres::Connection) -> Result<(), schemamama::Error<postgres::error::Error>> {
    let migrator = migrate(connection);
    return migrator.up(None);
}
