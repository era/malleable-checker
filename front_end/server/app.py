from flask import Flask
from flask import request
import os
import sqlite3

app = Flask(__name__)

db = None

@app.route("/")
def hello_world():
    return "<p>Hello, World!</p>"

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
   app.run(debug=True)
   db = sqlite3.connect(os.environ['SQLITE3_PATH'])