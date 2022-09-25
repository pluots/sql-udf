# UDF: MariaDB/MySQL User Defined Functions in Rust

This crate aims to make it extremely simple to implement UDFs for SQL, in a way
that is as safe as possible. 


# UDF Theory

Basic SQL UDFs consist of three exposed functions:

- An initialization function where arguments are checked and memory is allocated
- A processing function where a result is returned
- A deinitialization function where anything on the heap is cleaned up

This wrapper simplifies the process so that you only need to worry about
checking arguments and performing the task.

- Create a struct or enum that will be shared across initialization and
  processing (if there is nothing, just create an empty struct)
