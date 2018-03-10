# Database test

PostgreSQL database consistency test tool. Provides simple commend line interface to execute given queries on PostgreSQL server.

Queries are defined in test suites. Every suite represented with single YAML file.

## Usage

To start testing on PostgreSQL instance on localhost:6543:

```bash
./database-test -d DATABASE -u USERNAME -w PASSWORD -- example-suite.yaml ...
```

Optional arguments:

* `-h` (`--host-name`) HOSTNAME: PostgreSQL host name or IP address, default value - localhost;
* `-p` (`--port`) PORT: PostgreSQL port, default value - 5432;
* `-u` (`--user-name`) USERNAME: PostgreSQL user name;
* `-w` (`--password`) PASSWORD: PostgreSQL password;
* `-d` (`--database`) DATABASE: PostgreSQL database;
* `-n` (`--n-workers`) NWORKERS: Number of worker threads, default value - 4.

Also all possible arguments can be shown with `--help` option.

## Test suite file

Test suite file should be in YAML format. For example see `example.yaml` file in project root directory.

Test suite fields:

* `name`: string, name of this test case. Will be shown in execution log;
* `description`: optional string, description of test suite will be shown instead of name if given;
* `skip`: optional object, if defined will be used to check suite to be skipped (see Skip section below);
* `cases`: array of object, every object represents single test case (see Test Case section).

## Skip

Skip clause contains only two fields:

* `query`: string, query to execute;
* `n_rows`: object, contains criterion to check query result (see N Rows section).

If query result successfully passed check - a suite and all its test will be skipped. Otherwise all tests will start.

If query was executed with error. A test suite will be skipped. The execution error will be shown ion log.

## N Rows

Defines criterion to check number of query result rows. Contains two required fields:

* `condition`: string, condition to compare actual and exp acted number of rows. Can be one of [=, !=, <, >, <=, >=];
* `value`: integer, value to compare actual number of rows with.

## Test Case

Represents single test case. Every test case can be skipped like test suite. Test case contains following fields:

* `name`: string, test case name;
* `description`: optional string, description of test suite will be shown instead of name if given;
* `query`: string, query to execute for this test case;
* `n_rows`: optional object, if defined will be used to check number of rows in result set (see Skip section);
* `columns`: optional object, if defined will be used to check column values of every row in result set (see Columns section);

## Columns

Defines criterion to check values of every row in query result set. Contains three required fields:

* `condition`: string, condition to compare actual and expected number of rows. Can be one of [=, !=, <, >, <=, >=];
* `name`: string, column name to test.
* `value`: integer, value to compare actual data with. Can be integer, float or string.

Every row of query result set will be tested of this condition. If at least one row failed the test - whole test case will fail.

Value can be compared only with BIGINT (integer value), DOUBLE PRECISION (float value), VARCHAR (string value). If some column has different type it can be converted to one of these type using ::, `CAST` or `CONVERT` SQL functions. Example:

```sql
SELECT 1::INT8 AS integer_column, 1::FLOAT8 AS float_column;
```
