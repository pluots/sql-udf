# UDF: MariaDB/MySQL User Defined Functions in Rust

This crate aims to make it extremely simple to implement UDFs for SQL, in a
minimally error-prone fashion.

Looking for prewritten useful UDFs? Check out the UDF suite, which provides
downloadable binaries for some useful functions:
<https://github.com/pluots/udf-suite>.

View the docs here: <https://docs.rs/udf/latest>

## UDF Theory

Basic SQL UDFs consist of three exposed functions:

- An initialization function where arguments are checked and memory is allocated
- A processing function where a result is returned
- A deinitialization function where anything on the heap is cleaned up (performed
  automatically in this library)

Aggregate UDFs (those that work on more than one row at a time) simply need to
register two to three additional functions.

This library handles everything that used to be difficult about writing UDFs
(dynamic registration, allocation/deallocation, error handling, nullable values,
logging) and makes it _trivial_ to add any function to your SQL server instance.
It also inclues a mock interface, for testing your function implementation
without needing a server.

## Quickstartc

The steps to create a working UDF using this library are:

- Create a new rust project (`cargo new --lib my-udf`), add `udf` as a
  dependency (`cd my-udf; cargo add udf`) and change the crate type to a
  `cdylib` by adding the following to `Cargo.toml`:

  ```toml
  [lib]
  crate-type = ["cdylib"]
  ```

- Make a struct or enum that will share data between initializing and processing
  steps (it may be empty). The default name of your UDF will be your struct's
  name converted to snake case.
- Implement the `BasicUdf` trait on this struct
- Implement the `AggregateUdf` trait if you want it to be an aggregate function
- Add `#[udf::register]` to each of these `impl` blocks (optionally with a
  `(name = "my_name")` argument)
- Compile the project with `cargo build --release` (output will be
  `target/release/libmy_udf.so`)
- Load the struct into MariaDB/MySql using `CREATE FUNCTION ...`
- Use the function in SQL!

For an example of some UDFs written using this library, see either the
`udf-examples/` directory or the [`udf-suite`](https://github.com/pluots/udf-suite)
repository.

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
traits. See [the docs](https://docs.rs/udf/latest/udf/trait.BasicUdf.html)
for more information.

If you use rust-analyzer with your IDE, it can help you out. Just type
`impl BasicUdf for MyStruct {}` and place your cursor between the brackets -
it will offer to autofill the function skeletons (`ctrl+.` or `cmd+.`
brings up this menu if it doesn't show up by default).

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
types (GATs) which are only available on rust >= 1.65. This version only just
became stable (2022-11-03), so be sure to run `rustup update` if you run into
compiler issues.

CI runs tests on both Linux and Windows, and this crate should work for either.
MacOS is untested, but will likely work as well.

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

Testing in Docker is highly recommended, so as to avoid disturbing a host SQL
installation. See [the udf-examples readme](udf-examples/README.md) for
instructions on how to do this.


## Examples

The `udf-examples` crate contains examples of various UDFs, as well as
instructions on how to compile them. See [the readme](udf-examples/README.md)
there.


## Logging & Debugging Note

If you need to log things like warnings during normal use of the function,
anything printed to `stderr` will appear in the server logs (which can be viewed
with e.g. `docker logs mariadb_udf_test` if testing in Docker). The `udf_log!`
macro will print a message that matches the formatting of other SQL log
information. You can also enable the crate features `logging-debug` for function
entry/exitpoint debugging, or `logging-debug-calls` for information on the exact
call parameters from the MariaDB/MySQL server.

The best way to debug is to use the `udf::mock` module to create s.all unit
tests. These can be run to validate correctness, or stepped through with a
debugger if needed (this use case is likely somewhat rare). All types implement
`Debug` so they can also be easily printed (the builtin `dbg!` macro prints to
`stderr`, so this will also appear in logs):

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

## License

This work is dual-licensed under Apache 2.0 and GPL 2.0 (or any later version)
as of version 0.5.1. You can choose either of them if you use this work.

`SPDX-License-Identifier: Apache-2.0 OR GPL-2.0-or-later`
