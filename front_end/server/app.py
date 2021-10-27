from flask import Flask, render_template, request
from configparser import ConfigParser
import pathlib
import os
import sqlite3

app = Flask(__name__)

db = None

@app.route("/")
def hello_world():
    return render_template("index.html")

@app.route("/checkers")
def checkers():
    return "<p>Hello, World!</p>"

@app.route("/checkers/new")
def new_checkers():
    return "<p>Hello, World!</p>"

@app.route("/checkers/<id>")
def edit_checkers():
    return "<p>Hello, World!</p>"

@app.route("/datasources")
def dataspirces():
    return "<p>Hello, World!</p>"

@app.route("/datasources/<id>")
def edit_datasources():
    return "<p>Hello, World!</p>"

@app.route("/datasources/new")
def new_datasources():
    return render_template("datasource_form.html")

@app.route('/api/checker', methods = ['POST'])
def create_checker():
    """ Create a checker based on a JSON request as:
    {desc: str, datasources: int[]}
    """

    create_checker_sql = ''' INSERT INTO checker(desc, status)
              VALUES(?, 'GREEN') '''
    assign_checker_ds_sql = "INSERT INTO CHECKER_DATASOURCE(checker_id, datasource_id) VALUES (?, ?)"
    
    body = request.get_json(force=True)

    cur = db.cursor()
    cur.execute(create_checker_sql, body['desc'])
    checker_id = cur.lastrowid

    for datasource in body['datasources']:
        cur.execute(assign_checker_ds_sql, [checker_id, datasource])
    
    db.commit()

    return {'status': 'OK', 'id': checker_id}

@app.route('/api/datasource', methods = ['POST'])
def create_datasource():
    pass


if __name__ == '__main__':
    config_object = ConfigParser()
    
    # TODO stop assuming relative path
    path = str(pathlib.Path().absolute()) + "/"

    config_object.read(path + os.environ['CONFIG'])

    db = sqlite3.connect(path + config_object['CHECKER']['SQLITE_PATH'])

    app.run(host='0.0.0.0', debug=True)