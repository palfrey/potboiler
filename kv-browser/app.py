from flask import Flask, render_template, request
from flask_sqlalchemy import SQLAlchemy, BaseQuery
import os

app = Flask(__name__)
app.config["SQLALCHEMY_DATABASE_URI"] = os.environ['DATABASE_URL']
db = SQLAlchemy(app)

def schema(table_name, crdt_type):
    if crdt_type == "LWW":
        return [db.Table(table_name,
            db.Column('key', db.String, primary_key=True),
            db.Column('value', db.String),
            db.Column('crdt', db.String),
            extend_existing=True
        )]
    elif crdt_type == "ORSET":
        return [db.Table(table_name,
            db.Column('key', db.String, primary_key=True),
            db.Column('crdt', db.String),
            extend_existing=True
        ),
        db.Table("%s_items" % table_name,
            db.Column('collection', db.String, primary_key=True),
            db.Column('key', db.String, primary_key=True),
            db.Column('item', db.String, primary_key=True),
            db.Column('metadata', db.String),
            extend_existing=True
        )]
    else:
        raise Exception(crdt_type)

@app.route("/")
def index():
    table = request.args.get("table", default="_config")
    config = schema("_config", "LWW")
    if table == "_config":
        Table = config
        crdt = "LWW"
    else:
        config = config[0]
        row = db.session.execute(config.select().where(config.c.key==table)).fetchone()
        crdt = row["value"]["crdt"]
        Table = schema(table, crdt)
    data = [db.session.execute(t.select()) for t in Table]
    if crdt == "ORSET":
        extra = {}
        for x in data[1]:
            if x.collection not in extra:
                extra[x.collection] = {}
            extra[x.collection][x.item] = x.metadata
    else:
        extra = None
    return render_template("%s.html" % crdt, table=data[0], table_name=table, extra=extra)

if __name__ == "__main__":
    app.run()
