[![Coverage Status](https://coveralls.io/repos/ihrwein/actiondb/badge.svg?branch=master&service=github)](https://coveralls.io/github/ihrwein/actiondb?branch=master)

# actiondb

Actiondb is a library and its associated tools to efficiently extract information from unstructured data. It's a tool
to parse logs and extract key-value pairs with predefined patterns from them.

## Patterns

A pattern is composed of literals and parsers, like:

```
Jun %{INT:day} %{INT:hour}:%{INT:min}:%{INT:sec} server sshd[%{INT:pid}]: Accepted publickey for joe
```

It can be used to parse the following log message:

```
Jun 25 14:09:58 server sshd[26665]: Accepted publickey for joe
```

### Parsers

Parsers can be used to extract data from unstructured text.

Every parser has the following syntax:

```
%{PARSER_TYPE(required_arg1, required_arg2, optional_arg1="value", optional_arg2=value):parser_instance_name}
```

If a parser doesn't have extra arguments its parameter list can be omitted:

```
%{PARSER_TYPE:parser_instance_name}
```

#### Available parsers

#### SET

Parses only the characters which was given as its arguments. An optional
minimum and maximum length can be specified.

##### Example

```
%{SET("abcd",min_len=1,max_len=2):parsed_value_name}
```

It's identical to the `[abcd]{1,2}` regular expression.

#### INT

It reuses the `SET` parser with the character set of the numbers from `0` to
`9`. An optional minimum and maximum length can be specified.

#### GREEDY

It tries to fill in the gap between a parser and a literal or two literals. It will use
the following literal as an "end string" condition.

##### Example

Pattern:
```
from %{GREEDY:ipaddr}: %{INT:dunno}
```
Sample message:
```
from 1.2.3.4: 123
```
Extracted key-value pairs:
* `(ipaddr,1.2.3.4)`
* `(dunno,123)`

### adbtool

`adbtool` is a program which can be used for the following purposes:
* validate patterns
* parse unstructured texts

## Rust things

### Run only one test without muting stdout

```
cargo test -- --nocapture matcher::trie::node::node::given_empty_trie_when_literals_are_inserted_then_they_can_be_looked_up
```

### You need to move out a resource from &mut self

You can do this by destructoring it via a `let` binding. The destructoring
function (like `split()`) takes `self`, not a reference. Then it can destructor
it.

### Reference has a longer lifetime than the data it references

You can extend a lifetime with the following syntax:

```rust
struct LiteralLookupHit<'a, 'b: 'a, 'c>(&'a mut Node<'b>, &'c str);
```

### Cannot borrow mutable X multiple times, but there is only one active borrow
Check your lifetimes. If there is one on `self` it can cause problems. If a trait
doesn't need a litetime just one of its methods, then place the lifetime on the method
and not on the trait.

### Trait implementing a trait

Use case: You have a trait (`HasOptionalParameter`) which should be implemented
by all `Parser` implementations. At first they would be the same, so you decide
to implement `HasOptionalParameter` for `Parser` (`impl HasOptionalParameter for Parser`).
With this approach you can get weird compiler errors (`type X does not implement any method
  in scope named Y`, but the trait is in scope...).

The solution is to implement `HasOptionalParameter` for every type which implements `Parser`:

```
impl<T> HasOptionalParameter for T where T:Parser { ... }
```
