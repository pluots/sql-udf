# UDF: MariaDB/MySQL User Defined Functions in Rust

This crate aims to make it extremely simple to implement UDFs for SQL, in a way
that is as safe as possible. 


## UDF Theory

Basic SQL UDFs consist of three exposed functions:

- An initialization function where arguments are checked and memory is allocated
- A processing function where a result is returned
- A deinitialization function where anything on the heap is cleaned up

This wrapper greatly simplifies the process so that you only need to worry about
checking arguments and performing the task.

## Quickstart

A quick overview of the workflow process is:

- Make a struct or enum that will share data between initializing and processing
  steps (it may be empty). The name of this struct will be the name of your
  function (converted to snake case).
- Implement the `BasicUdf` trait on this struct
- Implement the `AggregateUdf` trait if you want it to be an aggregate function
- Add `#[udf::register]` above each of these `impls`
- Compile the project as a cdylib (output should be a `.so` file)
- Load the struct into MariaDB/MySql

## Detailed overview

This section goes into the details of implementing a UDF with this library, but
it is non-exhaustive. For that, see the documentation, or the `udf/examples`
directory for well-annotated examples.

### Struct creation

The first step is to create a struct (or enum) that will be used to share data
between all relevant SQL functions. These include:

- `init` Called once per result set. Here, you can store const data to your
  struct (if applicable)
- `process` Called once per row (or per group for aggregate functions). This
  function uses data in the struct and in the current row's arguments 
- `clear` Aggregate only, called once per group at the beginning. Reset the
  struct as needed.
- `add` Aggregate only, called once per row within a group. Perform needed
  calculations and save the data in the struct.

It is quite possible, especially for simple functions, that there is no data
that needs sharing. In this case, just make an empty struct and no allocation
will take place.

There is a bit of a caveat for functions returning strings; 

```rust
/// Function `sum_int` just adds all arguments as integers
struct SumInt {}

/// Function `avg_float` is an aggregate function.
struct AvgFloat {
    running_total: f64
}

/// Generate random lipsum
struct Lipsum {
    res: String
}
```

### BasicUdf Implementation

The next step is to implement the `BasicUdf` trait

```rust
#[register]
impl BasicUdf for SumInt {
    type Returns<'a> = Option<i64>;

    fn init<'a>(args: &'a ArgList<'a, Init>) -> Result<Self, String> {
        // ...
    }

    fn process<'a>(
        &'a mut self,
        args: &ArgList<Process>,
    ) -> Result<Self::Returns<'a>, ProcessError> {
        // ...
    }
}
```

### AggregateUdf Implementation

### Compiling

Assuming the above has been followed, all that is needed is to produce a C
dynamic library for the project. This can be done by specifying
`crate-type = ["cdylib"]` in your `Cargo.toml`. After this, compiling with
`cargo build --release` will produce a loadable `.so` file (in
`target/release`).

### Running

Once compiled, the produced object file needs to be copied to the location of
the `plugin_dir` SQL variable - usually, this is `/usr/lib/mysql/plugin/`.

Once that has been done, `CREATE FUNCTION` can be used in MariaDB/MySql to load
it.

```bash
cp /home/target/release/examples/libbasic_sum.so /usr/lib/mysql/plugin/
```

```sql
CREATE FUNCTION sum_int RETURNS integer SONAME 'libbasic_sum.so';
```

```
MariaDB [(none)]> select sum_int(1, 2, 3, 4, 5, 6, '1');
+--------------------------------+
| sum_int(1, 2, 3, 4, 5, 6, '1') |
+--------------------------------+
|                             22 |
+--------------------------------+
1 row in set (0.001 sec)
```

It is also quite possible to have more than one function in the same object
file.

## Building in Docker

These currently rely on a feature called generic associated types (GATs) which
are not currently available on stable. For that reason, rust version >= 1.65 is
required - this includes the current beta and nightly channels, and scheduled to
become stable on 2022-11-03.

```sh
docker run --rm -it -v $(pwd):/home rustlang/rust:nightly \
  bash -c "cd home; cargo build --release --example basic_sum"
```
