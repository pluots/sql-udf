# Basic UDF Examples

This crate contains various simple examples for various UDFs.

## Building

### Building for your local OS

If you are building for your local SQL server, `cargo build` will create the
correct library. Just copy the output to your plugin directory.

```bash
cargo build -p udf-examples --release
```

### Building for Linux/Docker on a non-linux OS

If you do not develop in Linux but require Linux binaries (e.g. to target
Docker), the project can be built in a docker image:

```sh
docker run --rm -it \
  -v "$(pwd):/build" \
  -e CARGO_HOME=/build/.docker-cargo \
  rustlang/rust:nightly \
  bash -c "cd /build; cargo build -p udf-examples --release"
```

## Running & Testing

It is highly recommended to always test your functions in a docker container,
since minor mistakes can crash your server (e.g. loading a different `.so` file
with the same name). To do this, you can run the following in your host OS:

```bash
# Add -d to the arguments if you don't want to keep the window open
docker run --rm -it  \
  -v $(pwd)/target:/target \
  -e MARIADB_ROOT_PASSWORD=banana \
  --name mariadb_udf_test \
  mariadb
```

This creates a container named `mariadb_udf_test` that has your `target/`
directory mounted at `/target`. So, in a separate host terminal, you can enter a
shell in the container:

```bash
docker exec -it mariadb_udf_test bash
```

Then copy the built & mounted `.so` file, and log in to your server:

```bash
cp /target/release/libudf_examples.so /usr/lib/mysql/plugin/

mysql -pbanana
```

Once logged in, you can load all available example functions:

```sql
CREATE FUNCTION is_const RETURNS string SONAME 'libudf_examples.so';
CREATE FUNCTION lookup6 RETURNS string SONAME 'libudf_examples.so';
CREATE FUNCTION sum_int RETURNS integer SONAME 'libudf_examples.so';
CREATE FUNCTION udf_sequence RETURNS integer SONAME 'libudf_examples.so';
CREATE FUNCTION lipsum RETURNS string SONAME 'libudf_examples.so';
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

MariaDB [db]> create table t1 (
    ->     id int not null auto_increment,
    ->     qty int,
    ->     cost real,
    ->     class varchar(30),
    ->     primary key (id)
    -> );
Query OK, 0 rows affected (0.016 sec)

MariaDB [db]> insert into t1 (qty, cost, class) values
    ->     (10, 50, "a"),
    ->     (8, 5.6, "c"),
    ->     (5, 20.7, "a"),
    ->     (10, 12.78, "b"),
    ->     (6, 7.2, "c"),
    ->     (2, 10.3, "b"),
    ->     (3, 9.1, "c");
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

```
