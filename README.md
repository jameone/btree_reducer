# Binary Tree Reducer (btree_reducer)

[![CodeBuild]][CodeBuild]
[![Version badge]][crates.io]
[![Docs badge]][docs.rs]

[CodeBuild]: https://codebuild.us-east-1.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoiSnZBOG1xd1FpMkRGSWM4M0dINFFudWZhM0NhaXdkR3V6YzIyc2FwK3hpWmZRcytvdHlMeDFXL3NKUTBnK3RsclY2aXo4NDFwNVVqbiszWUtObTk3cWFFPSIsIml2UGFyYW1ldGVyU3BlYyI6Ii9mQTBQN1cyTnd4NklZeGIiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=main
[Version badge]: https://img.shields.io/crates/v/btree_reducer
[crates.io]: https://crates.io/crates/btree_reducer
[Docs badge]: https://img.shields.io/badge/docs.rs-rustdoc-blue
[docs.rs]: https://docs.rs/btree_reducer/

This library presents the user with a high level data
structure called `BTreeReducer`. This data structure can
be thought of as a generalization of a Boolean logic gate.
The implementation employs `BTreeDAG` to construct a directed
acyclic graph (DAG) of "Contacts" (think switches, these can be
either normally open or normally closed, essentially XOR gates).
There is only ever one (1) root node (node 0) which represents
the output bit. All other nodes have an output dependent on
their input, state, program bits, and whether they have child
elements. If a node does not have child elements, i.e. is a
leaf node, then it is considered an input of the `BTreeReducer`.
That is, the number of inputs related to the `BTreeReducer`
is equal to the number of leaf nodes in the `BTreeReducer`'s DAG.

Each node consists of three (3) bits. One (1) input bit,
one (1) state bit, and one (1) program bit. If the node has
child elements, then the input bit is a function of the state
bit, program bit, and output of child elements defined by the
following pseudo code:

```yaml
Output:
  If state bit = 0:
    If program bit = 0:
      return: logical AND of all child element outputs
    If program bit = 1:
      return: logical OR of all child element outputs
  If state bit = 1:
    If program bit = 0:
      return: NOT logical AND of all child element outputs
    If program bit = 1:
      return: NOT logical OR of all child element outputs
```

For leaf nodes, the input bits are set manually via
the `transition_state` method which takes a state vector
where elements are in {0, 1} and having length equal to
the number of leaf nodes. Insertion order is preserved and
so the *n<sup>th</sup>* boolean element of the input state
vector corresponds to the *n<sup>th</sup>* gate relative to
order they were inserted. See the truth table below for the
output of leaf nodes.

| Program  | Input         | State          | Output        |
| :------: | :-----------: | :------------: | :-----------: |
| 0        | 0             | 0              | 0             |
| 0        | 0             | 1              | 1             |
| 0        | 1             | 0              | 1             |
| 0        | 1             | 1              | 0             |
| 1        | 0             | 0              | 1             |
| 1        | 0             | 1              | 0             |
| 1        | 1             | 0              | 0             |
| 1        | 1             | 1              | 1             |

## Example
As an example we will construct an XOR gate using the `BTreeReducer` struct.
By transitioning the input via an input string we will demonstrate
the XOR truth table.

We will then transition the state of the `BTreeReducer` struct via a state
string and demonstrate the same inputs now resolve an output that represents
the XNOR truth table.

This example involves another important concept, shorting, which is analogous to
short circuiting a physical input on an arrangement of digital logic gates.
This is achieved via the `short` method. With respect to the implementation,
this method is a wrapper around `BTreeDAG`'s `add_edge` method. The build-in
resolution algorithm will simply resolve the short as it does any other edge.

```rust
use btree_reducer::{BTreeReducer};
use alloc::vec::Vec;
use alloc::string::String;

fn main() {
    let mut reducer: BTreeReducer<bool> = BTreeReducer::new();
    let series_0 = reducer.add_contact(reducer.root());
    let parallel_1 = reducer.add_contact(series_0.clone());
    let series_1 = reducer.add_contact(series_0.clone());
    let input_0 = reducer.add_contact(parallel_1.clone());
    let input_1 = reducer.add_contact(parallel_1.clone());
    reducer.short(series_1.clone(), input_0)?;
    reducer.short(series_1, input_1)?;

    let ps: String = String::from("010100");
    reducer.reprogram(ps)?;

    let ss: String = String::from("000100");
    reducer.reconfigure(ss)?;

    // 00 -> 0
    let is: String = String::from("00");
    reducer.reinput(is)?;

    let input: String = reducer.input();
    assert_eq!(input.as_str(), "00");

    let configuration: String = reducer.configuration();
    assert_eq!(configuration.as_str(), "000100");

    let output: String = reducer.output();
    assert_eq!(output, "0");

    // 10 -> 1
    let is: String = String::from("10");
    reducer.reinput(is)?;

    let input: String = reducer.input();
    assert_eq!(input.as_str(), "10");

    let configuration: String = reducer.configuration();
    assert_eq!(configuration.as_str(), "000100");

    let output: String = reducer.output();
    assert_eq!(output, "1");

    // 01 -> 1
    let is: String = String::from("01");
    reducer.reinput(is)?;

    let input: String = reducer.input();
    assert_eq!(input.as_str(), "01");

    let configuration: String = reducer.configuration();
    assert_eq!(configuration.as_str(), "000100");

    let output: String = reducer.output();
    assert_eq!(output, "1");

    // 11 -> 0
    let is: String = String::from("11");
    reducer.reinput(is)?;

    let input: String = reducer.input();
    assert_eq!(input.as_str(), "11");

    let configuration: String = reducer.configuration();
    assert_eq!(configuration.as_str(), "000100");

    let output: String = reducer.output();
    assert_eq!(output, "0");

    // XOR -> XNOR
    let ss: String = String::from("100100");
    reducer.reconfigure(ss)?;

    // 00 -> 1
    let is: String = String::from("00");
    reducer.reinput(is)?;

    let input: String = reducer.input();
    assert_eq!(input.as_str(), "00");

    let configuration: String = reducer.configuration();
    assert_eq!(configuration.as_str(), "100100");

    let output: String = reducer.output();
    assert_eq!(output, "1");

    // 10 -> 0
    let is: String = String::from("10");
    reducer.reinput(is)?;

    let input: String = reducer.input();
    assert_eq!(input.as_str(), "10");

    let configuration: String = reducer.configuration();
    assert_eq!(configuration.as_str(), "100100");

    let output: String = reducer.output();
    assert_eq!(output, "0");

    // 01 -> 0
    let is: String = String::from("01");
    reducer.reinput(is)?;

    let input: String = reducer.input();
    assert_eq!(input.as_str(), "01");

    let configuration: String = reducer.configuration();
    assert_eq!(configuration.as_str(), "100100");

    let output: String = reducer.output();
    assert_eq!(output, "0");

    // 11 -> 1
    let is: String = String::from("11");
    reducer.reinput(is)?;

    let input: String = reducer.input();
    assert_eq!(input.as_str(), "11");

    let configuration: String = reducer.configuration();
    assert_eq!(configuration.as_str(), "100100");

    let output: String = reducer.output();
    assert_eq!(output, "1");
}
```

As a further example of the generality of this data structure, consider creating a simple
reducer which checks if any three (3) letter words contain vowels. Notice we are overriding
a couple of the traits to achieve the desired functionality.

```rust
use btree_reducer::{BTreeReducer, Transition, Output, Contact};
use alloc::vec::Vec;

fn main() {
    impl Transition<char> for char {
        fn transition(&self) -> char {
            'y'
        }
    }

    impl Output<char> for Contact<char> {
        type Error = Error;
        fn output(&mut self) -> char {
            let mut vowels: BTreeSet<char> = BTreeSet::new();
            vowels.insert('a');
            vowels.insert('e');
            vowels.insert('i');
            vowels.insert('o');
            vowels.insert('u');
            vowels.insert('y');
            if vowels.contains(&self.input) {
                'y'
            } else {
                'n'
            }
        }
    }

    let mut reducer: BTreeReducer<char> = BTreeReducer::new();
    reducer.add_contact(reducer.root());
    reducer.add_contact(reducer.root());
    reducer.add_contact(reducer.root());

    let mut input: Vec<char> = Vec::new();
    input.push('f');
    input.push('o');
    input.push('x');
    reducer.reinput(input.clone())?;

    let mut program: Vec<char> = Vec::new();
    program.push('n');
    program.push('\0');
    program.push('\0');
    program.push('\0');
    reducer.reprogram(program)?;

    assert_eq!(reducer.output(), 'y');

    let mut input: Vec<char> = Vec::new();
    input.push('c');
    input.push('a');
    input.push('t');
    reducer.reinput(input.clone())?;

    assert_eq!(reducer.output(), 'y');

    let mut input: Vec<char> = Vec::new();
    input.push('p');
    input.push('s');
    input.push('m');
    reducer.reinput(input.clone())?;

    assert_eq!(reducer.output(), 'n');

    let mut input: Vec<char> = Vec::new();
    input.push('i');
    input.push('b');
    input.push('m');
    reducer.reinput(input.clone())?;

    assert_eq!(reducer.output(), 'y');

    let mut input: Vec<char> = Vec::new();
    input.push('d');
    input.push('o');
    input.push('g');
    reducer.reinput(input.clone())?;

    assert_eq!(reducer.output(), 'y');

    let mut input: Vec<char> = Vec::new();
    input.push('t');
    input.push('l');
    input.push('s');
    reducer.reinput(input.clone())?;

    assert_eq!(reducer.output(), 'n');
}
```

## Usage

Add the following to your `Cargo.toml` file:
```toml
[dependencies]
btree_reducer = "0.1.0"
```

## API

Please see the [API](src/reducer/api.rs) for a full list of
available methods.

## License

This work is dually licensed under MIT OR Apache-2.0.