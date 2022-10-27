# UDF: MariaDB/MySQL User Defined Functions in Rust

This crate aims to make it extremely simple to implement UDFs for SQL, in a
minimally error-prone fashion.


## UDF Theory

Basic SQL UDFs consist of three exposed functions:

- An initialization function where arguments are checked and memory is allocated
- A processing function where a result is returned
- A deinitialization function where anything on the heap is cleaned up

This wrapper greatly simplifies the process so that you only need to worry about
checking arguments and performing the task.

There are also aggregate UDFs, which simply need to register two to three
additional functions.

## Quickstart

A quick overview of the workflow process is:

- Make a struct or enum that will share data between initializing and processing
  steps (it may be empty). The name of this struct will be the name of your
  function in SQL (converted to snake case).
- Implement the `BasicUdf` trait on this struct
- Implement the `AggregateUdf` trait if you want it to be an aggregate function
- Add `#[udf::register]` to each of these `impl` blocks
- Compile the project as a cdylib (output should be a `.so` file)
- Load the struct into MariaDB/MySql using `CREATE FUNCTION ...`
- Use the function in SQL

## Detailed overview

This section goes into the details of implementing a UDF with this library, but
it is non-exhaustive. For that, see the documentation, or the `udf-examples`
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
- `remove` Window functions only, called to remove a value from a group

It is quite possible, especially for simple functions, that there is no data
that needs sharing. In this case, just make an empty struct and no allocation
will take place.


```rust
/// Function `sum_int` just adds all arguments as integers and needs no shared data
struct SumInt;

/// Function `avg` on the other hand may want to save data to perform aggregation
struct Avg {
    running_total: f64
}
```

There is a bit of a caveat for functions returning buffers (string & decimal
functions): if there is a possibility that string length exceeds
`MYSQL_RESULT_BUFFER_SIZE` (255), then the string to be returned must be
contained within the struct (the `process` function will then return a
reference).

```rust
/// Generate random lipsum that may be longer than 255 bytes
struct Lipsum {
    res: String
}
```

### Trait Implementation

The next step is to implement the `BasicUdf` and optionally `AggregateUdf`
traits. See the docs for more information.

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

### Compiling

Assuming the above has been followed, all that is needed is to produce a C
dynamic library for the project. This can be done by specifying
`crate-type = ["cdylib"]` in your `Cargo.toml`. After this, compiling with
`cargo build --release` will produce a loadable `.so` file (located in
`target/release`).

Important version note: this crate relies on a feature called generic associated
types (GATs) which are only available on rust >= 1.65. At time of writing, this
is not yet stable (scheduled stable date is 2022-11-03), so make sure you are
using either the beta or nightly compiler to build anything that uses this
crate.

### Symbol Inspection

If you would like to verify that the correct C-callable functions are present,
you can inspect the dynamic library with `nm`. 

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

### Usage

Once compiled, the produced object file needs to be copied to the location of
the `plugin_dir` SQL variable - usually, this is `/usr/lib/mysql/plugin/`.

Once that has been done, `CREATE FUNCTION` can be used in MariaDB/MySql to load
it.


## Docker Use

If you require a linux object file but are compiling on a different platform,
building in docker is a convenient option:

```sh
# This will mount your current directory at /build, and use a new .docker-dargo
# directory for cargo's cache. This will share the `target/` directory
# Change the `bash -c` command based on what you want to build.
docker run --rm -it \
  -v "$(pwd):/build" \
  -e CARGO_HOME=/build/.docker-cargo \
  rustlang/rust:nightly \
  bash -c "cd /build; cargo build -p udf-examples --release"
```


### Testing in Docker

Testing in Docker is highly recommended, so as to avoid disturbing a host SQL
installation. See [the udf-examples readme](sql-udf/README.md) for instructions
on how to do this.


It can be convenient to test UDFs in a docker container:

```sh
# Start a mariadb server headless
# Mount our local target directory at /target
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

## Examples

The `udf-examples` crate contains examples of various UDFs, as well as
instructions on how to compile them. See [the readme](sql-udf/README.md)


## Logging & Debugging Note

If you need to log things like warnings during normal use of the function,
`eprintln!()` can be used to print to `stderr`. This will show up in the SQL
server logs; these can be viewed with e.g. `docker logs mariadb_udf_test` if
testing in `docker`.


The quickest way to do simple debugging is by using the `dbg!(...)` macro (rust
builtin). This also writes to `stderr` but prints file & line information and
the value of its argument (prettyprinted), and returns the argument for further
assignment or use.

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
