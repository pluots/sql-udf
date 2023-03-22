# Basic UDF Examples

This crate contains various simple examples for various UDFs.

## Building & Loading

### Docker

It is highly recommended to always test your functions in a docker container,
since minor mistakes can crash your server (e.g. loading a different `.so` file
with the same name).

Getting this going is easy for examples, because of the provided
`Dockerfile.examples` file. Just run the following:

```shell
docker build -f Dockerfile.examples . --tag mdb-example-so
```

Depending on your Docker version, you may need to prepend `DOCKER_BUILDKIT=1` to
that command (the Dockerfile leverages cache only available with buildkit).

Once built, you can start a container:

```bash
docker run --rm -d -e MARIADB_ROOT_PASSWORD=example --name mariadb_udf_test mdb-example-so
```

This will start it in headless mode. If you need to stop it, you can use
`docker stop mariadb_udf_test`. (Note that the server will delete its docker image
and thus its data upon stop, due to the `--rm` flag, so don't use this example
for anything permanent).

## Local OS

If you are building for your local SQL server, `cargo build` will create the
correct library. Just copy the resulting `.so` file (within the
`target/release/` directory) to your server's plugin directory.

```bash
cargo build -p udf-examples --release
```

## Testing

You will need to enter a SQL console to load the functions. This can be done
with the `mariadb`/`mysql` command, either on your host system or within the
docker container. If you used the provided command above, the password
is `example`.

```sh
docker exec -it mariadb_udf_test mysql -pexample
```

Note that this won't work immediately after launching the server, it takes a few
seconds to start.

Once logged in, you can load all available example functions:

```sql
CREATE FUNCTION is_const RETURNS string SONAME 'libudf_examples.so';
CREATE FUNCTION lookup6 RETURNS string SONAME 'libudf_examples.so';
CREATE FUNCTION sum_int RETURNS integer SONAME 'libudf_examples.so';
CREATE FUNCTION udf_sequence RETURNS integer SONAME 'libudf_examples.so';
CREATE FUNCTION lipsum RETURNS string SONAME 'libudf_examples.so';
CREATE FUNCTION log_calls RETURNS integer SONAME 'libudf_examples.so';
CREATE AGGREGATE FUNCTION avg2 RETURNS real SONAME 'libudf_examples.so';
CREATE AGGREGATE FUNCTION avg_cost RETURNS real SONAME 'libudf_examples.so';
CREATE AGGREGATE FUNCTION udf_median RETURNS integer SONAME 'libudf_examples.so';
```

And try them out!

```
MariaDB [(none)]> select sum_int(1, 2, 3, 4, 5, 6.4, '1');
+----------------------------------+
| sum_int(1, 2, 3, 4, 5, 6.4, '1') |
+----------------------------------+
|                               22 |
+----------------------------------+
1 row in set (0.000 sec)

MariaDB [(none)]> select is_const(2);
+-------------+
| is_const(2) |
+-------------+
| const       |
+-------------+
1 row in set (0.000 sec)

MariaDB [(none)]> select lookup6('localhost'), lookup6('google.com');
+----------------------+--------------------------+
| lookup6('localhost') | lookup6('google.com')    |
+----------------------+--------------------------+
| ::1                  | 2607:f8b0:4009:818::200e |
+----------------------+--------------------------+
1 row in set (0.027 sec)


-- Create table to test aggregate functions
MariaDB [(none)]> create database db; use db;
MariaDB [db]> create table t1 (
    id int not null auto_increment,
      qty int,
      cost real,
      class varchar(30),
      primary key (id)
    );
Query OK, 0 rows affected (0.016 sec)

MariaDB [db]> insert into t1 (qty, cost, class) values
    (10, 50, "a"),
    (8, 5.6, "c"),
    (5, 20.7, "a"),
    (10, 12.78, "b"),
    (6, 7.2, "c"),
    (2, 10.3, "b"),
    (3, 9.1, "c");
Query OK, 7 rows affected (0.007 sec)
Records: 7  Duplicates: 0  Warnings: 0

MariaDB [db]> select qty, udf_sequence() from t1 limit 4;
+------+----------------+
| qty  | udf_sequence() |
+------+----------------+
|   10 |              1 |
|    8 |              2 |
|    5 |              3 |
|   10 |              4 |
+------+----------------+
4 rows in set (0.000 sec)

MariaDB [db]> select avg_cost(qty, cost) from t1 group by class order by class;
+-------------------------+
| avg_cost(qty, cost)     |
+-------------------------+
| 40.23333333333333400000 |
| 12.36666666666666700000 |
|  6.78235294117647050000 |
+-------------------------+
3 rows in set (0.001 sec)

-- Check server logs after running this!
MariaDB [(db)]> select log_calls();

```

If you check your log files, you will notice that full call logging is enabled. You
can disable this by removing the `logging-debug` feature in the `udf-examples`
`Cargo.toml`.
