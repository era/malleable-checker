-- creates the minimum tables for checker to run, if you want you can add more columns to be used in the front end
CREATE TABLE IF NOT EXISTS CHECKER.CHECKER (id INTEGER AUTOINCREMENT, desc TEXT, status TEXT);
CREATE TABLE IF NOT EXISTS CHECKER.DATASOURCE (id INTEGER AUTOINCREMENT, name TEXT, code TEXT);
CREATE TABLE IF NOT EXISTS CHECKER.CHECKER_DATASOURCE (checker_id INTEGER, datasource_id INTEGER);