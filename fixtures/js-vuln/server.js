const express = require("express");
const { exec } = require("child_process");
const app = express();

// Intentionally vulnerable demo (DO NOT deploy)

app.get("/search", (req, res) => {
  const q = req.query.q;
  // SQL-ish concat
  const sql = `SELECT * FROM users WHERE name = '${q}'`;
  db.query(sql, (err, rows) => res.json(rows));
});

app.get("/page", (req, res) => {
  const name = req.query.name;
  res.send(`<div id="x"></div><script>document.getElementById('x').innerHTML = "${name}"</script>`);
});

app.get("/run", (req, res) => {
  exec(`echo ${req.query.cmd}`, (e, stdout) => res.send(stdout));
});

app.get("/eval", (req, res) => {
  res.send(String(eval(req.query.code)));
});

const https = require("https");
const agent = new https.Agent({ rejectUnauthorized: false });

app.listen(3000);
