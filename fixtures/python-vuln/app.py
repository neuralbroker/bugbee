from flask import Flask, request
import os
import subprocess
import pickle
import sqlite3

app = Flask(__name__)

# Intentionally vulnerable demo fixtures for Bugbee engines (DO NOT deploy).

@app.route("/search")
def search():
    q = request.args.get("q", "")
    conn = sqlite3.connect("app.db")
    cur = conn.cursor()
    # SQL injection sink
    cur.execute("SELECT * FROM items WHERE name = '%s'" % q)
    return str(cur.fetchall())

@app.route("/run")
def run_cmd():
    cmd = request.args.get("cmd", "echo hi")
    # command injection
    return os.system(cmd)

@app.route("/load", methods=["POST"])
def load_blob():
    data = request.get_data()
    # insecure deserialization
    return str(pickle.loads(data))

@app.route("/calc")
def calc():
    expr = request.args.get("e", "1+1")
    return str(eval(expr))

if __name__ == "__main__":
    # misconfig: debug true
    app.run(debug=True)
