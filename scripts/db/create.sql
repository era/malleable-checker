-- creates the minimum tables for checker to run, if you want you can add more columns to be used in the front end
CREATE TABLE IF NOT EXISTS CHECKER (id INTEGER primary key AUTOINCREMENT, desc TEXT, status TEXT);
CREATE TABLE IF NOT EXISTS DATASOURCE (id INTEGER primary key AUTOINCREMENT, name TEXT, code TEXT);
CREATE TABLE IF NOT EXISTS CHECKER_DATASOURCE (checker_id INTEGER, datasource_id INTEGER);
CREATE TABLE IF NOT EXISTS CHECKER_EXECUTION (checker_id INTEGER, run_at TEXT, status TEXT);