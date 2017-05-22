## CRUST
### C/C++ to Rust Transpiler

C++ is a system programming language widely used in development of operating system,  firmwares, device drivers and in some application development. Major drawback with C++ or any other system programming language is memory safety, null pointers and dangling pointers, which are very dangerous if not handled properly by the programmer. The new programming language *Rust* is system programming language provides the safe and secure programming with highly enforced compiler restrictions with zero cost abstraction.

------------------------------------------------------------------------

This tool is intended to translate existing C++ code base into Rust with less effort.
May require manual lookup or minute edit to the translated code.

------------------------------------------------------------------------

## Usage

First, make sure you are setup for Rust development. Check out [http://www.rust-lang.org](http://www.rust-lang.org) for more information. The installation sets up the Rust compiler and Cargo package management system. Also, it adds `rustc` and `cargo` commands to your PATH variable.

Now that you're setup for rust. Open any suitable terminal and `cd` into the crust directory

Run `cargo build` to compile the entire project and download some dependencies.

Now, you many test out CRUST using some of the examples in the example folder as follows:
`cargo run`

The program asks for the C/C++ file to be converted to Rust (Enter the path relative to the current location):
```
Enter the C/C++ file to be converted to Rust : examples/prog.cpp
```
Next, enter the translation mode, Strict or Loose (default is Loose). Strict mode assumes all variables to be immutable and Loose mode makes all variables mutable.

It also asks whether the program should be converted into a cargo project. It essentially sets up the program into a project with package management. Check out [http://doc.crates.io/](http://doc.crates.io/) for more information.
```
Enter the translation mode [(S/s)trict/(L/l)oose] : l
Do you want to create a cargo project :[Y/N]n
```

Now, it completes the translation and shows where the translated file is stored
```
Input file size : 177bytes
Tokenizing.......       :DONE
Invoking Parser ........        :DONE
Rust equivalent of source of `examples/prog.cpp` is generated successfully, View the rust code in file : ./examples/pro
g.rs
```
------------------------------------------------------------------------
Alternatively, you may generate an executable similar to C executable as follows:

`cargo build --release`

This generates an executable, `/target/release/crust` which can be moved anywhere and run anytime using:
`./crust`


------------------------------------------------------------------------


### Implementation details

On a broad scale, there are two parts to the Transpiler: The Lexer and The Parser. The Lexer is quite generic and is built primarily using Rust’s ‘match’ construct and regex which is an abstraction over a DFA. Since this part of the Transpiler is fairly common, there is not much to be said about it. It takes in syntactically correct C/C++ code, tokenizes it, creates lexemes and adds it to a symbol table. We would rather focus our attention on the second part of the Transpiler, which is the Parser.The Parser is a Recursive Descent Parser that has been completely redesigned for modularity, and ease of debugging and testing.

We believe this is a new concept in the context of design of parsers. It starts off by collecting the stream of lexemes generated by the Lexer. It feeds this into our recursive descent parser. The Parser has two parts that work hand-in-hand to convert the C/C++ lexemes into Rust lexemes: Global Construct Identifier (hereafter referred to as GCI) and several Nano-Parsers.

The GCI is a sort of control loop which is responsible for iterating through the input lexemes and recognizing various C constructs by matching them with predefined grammars. Once it successfully recognizes a construct, the GCI assigns the job of translation to one of the Nano-Parsers. It then collects the translated snippets from each Nano Parser and puts everything together .

We have devised a new and efficient method to construct compilers in what we call “The Nano-Parser Methodology”. This is described in detail in the next section.

Lastly, we have provided the user with two different translation schemes, which is handled by the Mutability Analyzer. Rust variables may either be mutable or immutable, depending on whether they need to be allowed to be modified. This is a safety feature of Rust to prevent accidental modification of variables that must not be modified. On attempting to mutate an immutable variable, compile-time errors are generated.

--------------------------------------------------------------------------------------

## The Nano Parser Methodology
The Nano-Parser Methodology used in the design of CRUST makes the parser elegant and simple. We define the method simply as follows: “A parser composed of several tiny, self-contained and well-defined parsers, capable of recognizing a single construct, all controlled by one main parser (GCI)”.

The GCI will identify a construct (through its grammar) from a given valid input and determine which Nano-Parser to call. As an example, if the GCI recognizes a “for” construct, it immediately calls the “for nano-parser”, which can parse only a “for” construct. Taking this one level deeper, if the “for” nano-parser finds a “if” construct during its parsing, it calls the “if nano-parser” and so on.

In CRUST, and indeed any parser, this recursion is an elegant definition, leading to recursive translation based on the given input. Counter intuitively, this is a far more efficient method than an iterative one.
There are several advantages that accompany the use of the Nano-Parser Methodology.

The code is quite modular and highly manageable since each nano-parser is defined in its own method. Writing tests for compilers are very often something that developers struggle with. The modular design helped immensely with this. We were able to cover about 95% of our code through unit tests. Bugs, if introduced in the parser are easy to isolate to a single method. As a language evolves, rewriting a parser to support this is cumbersome.

Our methodology makes this task quite simple, since a new construct can be handled by simple adding a compatible nano-parser and linking it with the GCI.
