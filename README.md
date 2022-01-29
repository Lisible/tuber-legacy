![tuber logo](img/tuber_logo.png)
# tuber
*Make your games taste like a Piemontese salad*

***tuber*** is a game engine I'm currently working on during my spare time.

# How to run the examples

You can run the examples by going into the ``examples/<example>`` directory and running the following example
  
```bash
cargo run --example <example>
```

# How to build the book

In order to build the book, you need to install [mdbook](https://rust-lang.github.io/mdBook/).

To build the book, navigate to the ``book`` directory use the following command
```bash
mdbook build
```
\
Similarly, to serve it on a local server use the following command
```bash
mdbook serve
```