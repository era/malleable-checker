from flask import Flask, render_template, request
from configparser import ConfigParser
import pathlib
import os
import sqlite3

#TODO seems like some version of Flask does not like when
# the response of a method is a dict and not a string
app = Flask(__name__,static_folder=os.path.dirname(__file__) + '/static')

db_path = None

@app.route("/")
def hello_world():
    return render_template("index.html")

@app.route("/checkers")
def checkers():

    db = sqlite3.connect(db_path)

    cur = db.cursor()
    cur.execute("SELECT id, desc, status FROM checker")

    checkers = cur.fetchall()

    return render_template("list_checkers.html", checkers=checkers)

@app.route("/checkers/new")
def new_checkers():
    db = sqlite3.connect(db_path)

    cur = db.cursor()
    cur.execute("SELECT name, id FROM datasource")

    datasources = cur.fetchall()

    return render_template("checker_form.html", datasources=datasources, checker=[], selected_datasources=[])

@app.route("/checkers/<id>")
def edit_checkers(id):
    db = sqlite3.connect(db_path)

    cur = db.cursor()
    cur.execute("SELECT name, id FROM datasource")

    datasources = cur.fetchall()

    cur.execute("SELECT id, desc FROM checker where id = ?", [id])

    checker = cur.fetchone()

    selected_datasources = cur.execute("SELECT datasource_id  FROM checker_datasource where checker_id = ?", [id])

    selected_datasources = [ds[0] for ds in selected_datasources]

    return render_template("checker_form.html", datasources=datasources, checker=checker, selected_datasources=selected_datasources)

@app.route("/datasources")
def datasources():
    db = sqlite3.connect(db_path)

    cur = db.cursor()
    cur.execute("SELECT name, code, id FROM datasource")

    datasources = cur.fetchall()

    return render_template("list_datasource.html", datasources=datasources)

@app.route("/datasources/<id>")
def edit_datasources(id):
    db = sqlite3.connect(db_path)

    sql = "SELECT name, code, id from datasource where id = ?"

    cur = db.cursor()

    cur.execute(sql, [id])

    return render_template("datasource_form.html", ds=cur.fetchone())

@app.route("/datasources/new")
def new_datasources():
    return render_template("datasource_form.html", ds=[])

@app.route('/api/checker/', methods = ['POST'])
def create_checker():
    """ Create a checker based on a JSON request as:
    {desc: str, datasources: int[]}
    """
    db = sqlite3.connect(db_path)

    create_checker_sql = ''' INSERT INTO checker(desc, status)
              VALUES(?, 'GREEN') '''
    assign_checker_ds_sql = "INSERT INTO CHECKER_DATASOURCE(checker_id, datasource_id) VALUES (?, ?)"
    
    body = request.get_json(force=True)

    cur = db.cursor()
    cur.execute(create_checker_sql, [body['desc']])
    checker_id = cur.lastrowid

    for datasource in body['datasources']:
        cur.execute(assign_checker_ds_sql, [checker_id, datasource])
    
    db.commit()

    return {'status': 'OK', 'id': checker_id}

@app.route('/api/datasource/', methods = ['POST'])
def create_datasource():
    """ Create a datasource based on a JSON request as:
    {sql: str, name: str}
    """
    db = sqlite3.connect(db_path)
    create_datasource_sql = "INSERT INTO datasource(name, code) VALUES (?, ?)"
    body = request.get_json(force=True)

    cur = db.cursor()
    cur.execute(create_datasource_sql, [body['name'], body['sql']])

    id = cur.lastrowid

    db.commit()

    return {'status': 'OK', 'id': id}


@app.route('/api/datasource/<id>', methods = ['POST'])
def update_datasource(id):
    """ Update a datasource based on a JSON request as:
    {sql: str, name: str}
    """
    db = sqlite3.connect(db_path)
    update_datasource_sql = "UPDATE datasource set name = ?, code = ? where id = ?"
    body = request.get_json(force=True)

    cur = db.cursor()
    cur.execute(update_datasource_sql, [body['name'], body['sql'], id])

    db.commit()

    return {'status': 'OK', 'id': id}


if __name__ == '__main__':
    config_object = ConfigParser()

    config_object.read(os.environ['CONFIG'])
    db_path = config_object['CHECKER']['SQLITE_PATH']

    app.run(host='0.0.0.0', debug=True)