from flask import Flask, render_template, request
from flask_sqlalchemy import SQLAlchemy, BaseQuery
import os

app = Flask(__name__)
app.config["SQLALCHEMY_DATABASE_URI"] = os.environ['DATABASE_URL']
db = SQLAlchemy(app)

@app.route("/")
def index():
    table = request.args.get("table", default="_config")
    Table = db.Table(table,
        db.Column('key', db.String, primary_key=True),
        db.Column('value', db.String),
        db.Column('crdt', db.String),
        extend_existing=True
    )
    data = db.session.execute(Table.select())
    print(data)
    return render_template("table.html", table=data, table_name=table)

if __name__ == "__main__":
    app.run()
