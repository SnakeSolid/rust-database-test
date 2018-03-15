# Database test

PostgreSQL database consistency test tool. Provides simple commend line interface to execute given queries on PostgreSQL server.

Queries are defined in test suites. Every suite represented with single YAML file.

For using in automated testing the application returns different exit codes, see [Exit code](#markdown-header-exit-code) section.

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
* `-r` (`--recursive`): Read all files under each directory, recursively;
* `-e` (`--extensions`) EXTENSIONS: File extension filters for recursive search;
* `-f` (`--filter`) FILTER: Filter test suite by suite name;
* `-t` (`--text-mode`): Use plain text mode instead of color;
* `-n` (`--n-workers`) NWORKERS: Number of worker threads, default value - 4.

Also all possible arguments can be shown with `--help` option.

Extension filters used only for recursive directory processing. If parameter is file, it will be processed regardless its extension.

## Exit code

The application returns exit code for automated testing. There are three possible exit codes:

* `0` - all tests passed or skipped, no failed tests;
* `1` - at least one test failed;
* `2` - error occurred during startup (server refused connection, malformed YAML, etc).

## Test suite file

Test suite file should be in YAML format. For example see `example.yaml` file in project root directory.

Test suite fields:

* `name`: string, name of this test case. Will be shown in execution log;
* `description`: optional string, description of test suite will be shown instead of name if given;
* `skip`: optional object, if defined will be used to check suite to be skipped (see [Skip](#markdown-header-skip) section below);
* `cases`: array of object, every object represents single test case (see [Test Case](#markdown-header-test-case) section).

## Skip

Skip clause contains only two fields:

* `query`: string, query to execute;
* `n_rows`: object, contains criterion to check query result (see [N Rows](#markdown-header-n-rows) section).

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
* `n_rows`: optional object, if defined will be used to check number of rows in result set (see [Skip](#markdown-header-skip) section);
* `columns`: optional object, if defined will be used to check column values of every row in result set (see [Columns](#markdown-header-columns) section);

## Columns

Defines criterion to check values of every row in query result set. This object may be [simple condition](#markdown-header-condition), [range check](#markdown-header-range), [any value check](#markdown-header-any) or [substring check](#markdown-header-contains).

Every row of query result set will be tested of this condition. If at least one row failed the test - whole test case will fail.

Value can be compared only with BIGINT/INT8 (integer value), DOUBLE PRECISION/FLOAT8 (float value), VARCHAR/CHAR (string value). If some column has different type it can be converted to one of these type using `::`, `CAST` or `CONVERT` SQL functions. Example:

```sql
SELECT 1::INT8 AS integer_column, 1::FLOAT8 AS float_column;
```

### Condition

Simple condition has three required fields:

* `name`: string, column name to test;
* `condition`: string, condition to compare actual and expected number of rows. Can be one of [`=`, `!=`, `<`, `>`, `<=`, `>=`];
* `value`: integer/float/string, value to compare actual data with.

### Range

Range check has three required fields:

* `name`: string, column name to test;
* `from`: integer/float/string, represents minimal value for column;
* `to`: integer/float/string, represents maximal value for column.

The values of `from` and `to` must be same.

### Any

Any check has two required fields:

* `name`: string, column name to test;
* `any`: array of values. Can be integer, float or string.

Value of column will be compared with all values in `any` parameter. Match success if at least one value equals to actual value.

### Contains

Contains check has two required fields:

* `name`: string, column name to test;
* `contains`: string, substring to search in actual value.
