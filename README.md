PizzaML
=======

This is a very silly project to make Standard ML more appealing to various groups including:

* Systems programmers
* Rust programmers
* People born after 1973

At the moment this repository contains the skeleton of a transpiler from PizzaML's Rust-like
syntax to something vaguely resembling Standard ML.

Given this input file in PizzaML:

```rust
// examples/hello.pz
fn main() {
    print("Hello world!\n");
    print("We can print ;)\n");
}
```

We can compile it to SML as follows:

```
$ cargo build --release
$ ./target/release/pizza_ml examples/hello.pz
$ cat output.sml
```

The contents of `output.sml` should be something like the following:

```sml
fun main () = let
val _ = (TextIO.print "Hello world!\n");
val _ = (TextIO.print "We can print ;)\n");
in ()
end;
```

You can then compile this to a binary using a Standard ML compiler, e.g. PolyML

```
$ polyc output.sml -o output
$ ./output
```
