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

Additionally, there are aggregate UDF

## Quickstart

A quick overview of the workflow process is:

- Make a struct or enum that will share data between initializing and processing
  steps (it may be empty). The name of this struct will be the name of your
  function (converted to snake case).
- Implement the `BasicUdf` trait on this struct
- Implement the `AggregateUdf` trait if you want it to be an aggregate function
- Add `#[udf::register]` above each of these `impl` blocks
- Compile the project as a cdylib (output should be a `.so` file)
- Load the struct into MariaDB/MySql using `CREATE FUNCTION ...`
- Use the function in SQL

## Detailed overview

This section goes into the details of implementing a UDF with this library, but
it is non-exhaustive. For that, see the documentation, or the `udf_examples`
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


```rust
/// Function `sum_int` just adds all arguments as integers
struct SumInt;

/// Function `avg_float` may want to save data to perform aggregation
struct AvgFloat {
    running_total: f64
}
```


There is a bit of a caveat for functions returning buffers (string & decimal
functions): if there is a possibility that string length exceeds
`MYSQL_RESULT_BUFFER_SIZE` (255), then the string must be contained within the
struct. The `Returns` type would then be specified as `&'a [u8]`, `&'a str`, or
their `Option<...>` versions as applicable.

```rust
/// Generate random lipsum that may be longer than 255 bytes
struct Lipsum {
    res: String
}
```

### BasicUdf Implementation

The next step is to implement the `BasicUdf` trait

```rust
use udf::prelude::*;

struct SumInt;

#[register]
impl BasicUdf for SumInt {
    type Returns<'a> = Option<i64>;

    fn init<'a>(
      cfg: &UdfCfg<Init>,
      args: &'a ArgList<'a, Init>
    ) -> Result<Self, String> {
      // ...
    }

    fn process<'a>(
        &'a mut self,
        cfg: &UdfCfg<Process>,
        args: &ArgList<Process>,
        error: Option<NonZeroU8>,
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

Important version note: this crate relies on a feature called generic associated
types (GATs) which are only available on rust >= 1.65. At time of writing, this
is not yet stable (scheduled stable date is 2022-11-03), so make sure you are
using either the beta or nightly compiler to build anything that uses this
crate.

### Usage

Once compiled, the produced object file needs to be copied to the location of
the `plugin_dir` SQL variable - usually, this is `/usr/lib/mysql/plugin/`.

Once that has been done, `CREATE FUNCTION` can be used in MariaDB/MySql to load
it.

### Building & Running Examples

This repository contains a crate called `udf-example`, with a handful of example
functions. These can be built as follows:

```bash
cargo build -p udf-examples --release
cp target/release/libudf_examples.so /usr/lib/mysql/plugin/
```

Available symbols can always be inspected with `nm`:

```sh
# Output of example .so
$ nm -gC --defined-only target/release/libudf_examples.so
00000000000081b0 T avg_cost
0000000000008200 T avg_cost_add
00000000000081e0 T avg_cost_clear
0000000000008190 T avg_cost_deinit
0000000000008100 T avg_cost_init
0000000000009730 T is_const
0000000000009710 T is_const_deinit
0000000000009680 T is_const_init
0000000000009320 T sql_sequence
...
```

Load all available functions in SQL:

```sql
CREATE FUNCTION sum_int RETURNS integer SONAME 'libudf_examples.so';
CREATE FUNCTION sql_sequence RETURNS integer soname 'libudf_examples.so';
CREATE FUNCTION is_const RETURNS string soname 'libudf_examples.so';
CREATE AGGREGATE FUNCTION avg_cost RETURNS integer soname 'libudf_examples.so';
CREATE AGGREGATE FUNCTION udf_median RETURNS integer soname 'libudf_examples.so';
```

And try them out!

```
MariaDB [(none)]> select sum_int(1, 2, 3, 4, 5, 6, '1');
+--------------------------------+
| sum_int(1, 2, 3, 4, 5, 6, '1') |
+--------------------------------+
|                             22 |
+--------------------------------+
1 row in set (0.001 sec)
```


## Docker & Testing

If you require a linux object file but are compiling on a different platform,
building in docker is a convenient option:

```sh
# This will mount your current directory at /build, and use a new .docker-dargo
# directory for cargo's cache. It will use your same target folder (different )
# Change the `bash -c` command based on what you want to build.
docker run --rm -it \
  -v "$(pwd):/build" \
  -e CARGO_HOME=/build/.docker-cargo \
  rustlang/rust:nightly \
  bash -c "cd /build; cargo build -p udf-examples --release"
```

### Testing in docker

It can be convenient to test UDFs in a docker container. Here 

```sh
# Start a mariadb server headless
docker run --rm -it  \
  -v $(pwd)/target:/target \
  -e MARIADB_ROOT_PASSWORD=banana \
  --name mariadb_udf_test \
  mariadb

# Open a terminal in another window
docker exec -it mariadb_udf_test bash

# Copy output .so files
cp /target/release/libudf_examples.so /usr/lib/mysql/plugin/

# Log in with our password
mysql -pbanana
```

Run the `CREATE FUNCTION` commands specified above, then you will be able to
test the functions.

```sql
select sum_int(1, 2.2, '4');
# sequences work best with a table
select sql_sequence(1);
select udf_median(4);
```

### Debugging

The quickest way to debug is by using `dbg!()` or `eprintln!()`, which will show
up in server logs. `dbg!(...)` is usually preferred because it shows line
information, and lets you assign its contents:

```rust
dbg!(&self);
let arg0 = dbg!(args.get(0).unwrap())
```

```
[udf_examples/src/avgcost.rs:58] &self = AvgCost {
    count: 0,
    total_qty: 0,
    total_price: 0.0,
}

[udf_examples/src/avgcost.rs:60] args.get(0).unwrap() = SqlArg {
    value: Int(
        Some(
            10,
        ),
    ),
    attribute: "qty",
    maybe_null: true,
    arg_type: Cell {
        value: INT_RESULT,
    },
    marker: PhantomData<udf::traits::Process>,
}
```
