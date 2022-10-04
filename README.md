# UDF: MariaDB/MySQL User Defined Functions in Rust

This crate aims to make it extremely simple to implement UDFs for SQL, in a way
that is as safe as possible. 


## UDF Theory

Basic SQL UDFs consist of three exposed functions:

- An initialization function where arguments are checked and memory is allocated
- A processing function where a result is returned
- A deinitialization function where anything on the heap is cleaned up

This wrapper simplifies the process so that you only need to worry about
checking arguments and performing the task.

- Create a struct or enum that will be shared across initialization and
  processing (if there is nothing, just create an empty struct)

## Building in Docker

These currently rely on a feature called generic associated types (GATs)

```sh
docker run --rm -it -v $(pwd):/home rustlang/rust:nightly \
  bash -c "cd home; cargo build --release --example basic_sum"
```

## Running

`cp /home/target/release/examples/libbasic_sum.so /usr/lib/mysql/plugin/`
`create function sum_int returns integer soname 'libbasic_sum.so';`

```
MariaDB [(none)]> select sum_int(1, 2, 3, 4, 5, 6, '1');
+--------------------------------+
| sum_int(1, 2, 3, 4, 5, 6, '1') |
+--------------------------------+
|                             22 |
+--------------------------------+
1 row in set (0.001 sec)
```
