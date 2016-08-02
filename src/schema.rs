extern crate schemamama_postgres;
extern crate postgres;

use schemamama::Migrator;
use schemamama_postgres::{PostgresAdapter, PostgresMigration};

struct CreateLog;
migration!(CreateLog, 201605221254, "create log table");

impl PostgresMigration for CreateLog {
    fn up(&self, transaction: &postgres::Transaction) -> Result<(), postgres::error::Error> {
        transaction.execute(
            "CREATE TABLE log (id UUID PRIMARY KEY, owner UUID NOT NULL, \
            next UUID, prev UUID, data JSON NOT NULL);", &[])
            .unwrap();
        return Ok(());
    }

    fn down(&self, transaction: &postgres::Transaction) -> Result<(), postgres::error::Error> {
        let _ = transaction.execute("DROP TABLE log;", &[]).unwrap();
        return Ok(());
    }
}

fn migrate(connection: &postgres::Connection) -> Migrator<PostgresAdapter> {
    let adapter = PostgresAdapter::new(connection);
    let _ = adapter.setup_schema().unwrap();

    let mut migrator = Migrator::new(adapter);
    migrator.register(Box::new(CreateLog));
    return migrator;
}

pub fn up(connection: &postgres::Connection) -> Result<(), postgres::error::Error> {
    let migrator = migrate(connection);
    return migrator.up(migrator.last_version().unwrap());
}
