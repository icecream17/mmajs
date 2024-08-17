# Specification of the Metamath Language

This is <https://us.metamath.org/downloads/metamath.pdf> in a more codable form.

## Notation

> With inspiration from many sources; mostly <https://doc.rust-lang.org/reference/notation.html> and ECMA262; but also Python and C

* {{Syntactical production}}
* {Token}
* 'Character'
* `literal`
* g{globalVariable}
* :attributeOrMethod
* _localVariable_
* <span style="color:#AB5753;">**Error 73: Example**</span>

Productions or tokens can have associated _Behavior_ which is done upon completed parsing of the production or token, unless otherwise specified.

Errors are just suggestions; they can work however you want as long as you prepare grace and helpful output.

## Initialization

The following "global variables" are declared:

| type                       | name                       |
|----------------------------|----------------------------|
| List\<String>              | g{FilesIncluded} |
| List\<{{Scope}}>           | g{Scopes}         |
| Set\<{MathSymbol}>         | g{Constants}          |
| Set\<{MathSymbol}>         | g{MathSymbols}          |
| Set\<{MathSymbol}>         | g{InactiveVariables}          |
| List\<{{FHypothesis}}>     | g{FHypotheses}   |
| List\<({Label}, {{FHypothesis}} \| {{Assertion}})> | g{Labels}   |

1. Add the top level file name to g{FilesIncluded}.
1. Add a new scope to g{Scopes}.

## g:label(_label_: {Label})

1. For each let (_l_, _node_) in g{Labels}:
    1. If _l_ is _label_, return Some(_node_)
1. Return None

## g:addLabel(_label_: {Label}, _noun_)

1. If g:label(_label_) is not None, raise <span style="color:#AB5753;">**Error -1: Duplicate label**</span>
1. If g{MathSymbols} has _label_, raise <span style="color:#AB5753;">**Error -2: Label conflicts with math symbol**</span>
1. Append the entry (_label_, _noun_) to g{Labels}

## g:checkPossVarHasF(_variable_: {MathSymbol})

1. Assert: g{Scopes} is nonempty
1. If g{Scopes}:last:activeVariables contains _variable_,
    1. If g{Scopes}:last:activeFHypotheses does Not contain an {FHypothesis} whose
       :variable = _variable_, raise <span style="color:#AB5753;">**Error -3: Type of variable not defined yet**</span>

## List\<{MathSymbol}>

### :variables

1. Assert: g{Scopes} is nonempty
1. Return the set of _variable_ in **this** such that g{Scopes}:last:activeVariables contains _variable_

## Scope

A scope has the following attributes:

| type                           | name                       |
|--------------------------------|----------------------------|
| Set\<{MathSymbol}>             | :localVariableDeclarations |
|                                | :localDVConditions         |
| List\<{{FHypothesis}}>         | :localFHypotheses          |
| List\<{{EssentialHypothesis}}> | :localEssentialHypotheses  |

and the following methods:

### :parentScope

1. Let _index_ be the index of **this** scope in g{Scopes}
1. If _index_ is 0, return None
1. Return Some(g{Scopes}\[_index_ - 1])

### :activeVariables

1. If :parentScope is None, return :localVariableDeclarations
1. Return :localVariableDeclarations ++ :parentScope:activeVariables

> Note: ++ is the concatenation operation in Metamath

### :activeMathSymbols

1. Return :activeVariables ++ g{Constants}

### :activeDVConditions

1. If :parentScope is None, return :localDVConditions
1. Return :localDVConditions ++ :parentScope:activeDVConditions

### :activeDVPairs

1. Return every 2-size combination of variables in :activeDVConditions

### :activeFHypotheses

1. If :parentScope is None, return :localFHypotheses
1. Return :localFHypotheses ++ :parentScope:activeFHypotheses

### :activeEssentialHypotheses

1. If :parentScope is None, return :localEssentialHypotheses
1. Return :localEssentialHypotheses ++ :parentScope:activeEssentialHypotheses

### :whenDelete

1. g{InactiveVariables} ++= :localVariableDeclarations

## Top level productions

### {{Database}}

- {{Item}}<sup>*</sup>

> **Note**:
>
> Upon reaching the end of a file, a custom {EOF} token is added

> **Note**:
>
> {Whitespace} plays a special role in the lexer:
> whenever it is encountered, the current token ends,
> (with the exception of {CompressedProofNumber})
> and the rest of the whitespace is eaten.
>
> For convenience, a token will end if AND only if there is whitespace
> or the {EOF}
>
> If a token does not match any option, raise <span style="color:#AB5753;">**Error 0: Unexpected token**</span>

### {{Item}}

- {{Statement}}
- {{ConstantDeclaration}}
- {{FileInclusion}}
- {`EOF`}

> **Note**:
>
> {{Comment}} is moved inside {{Statement}}

> **Note**:
>
> {`EOF`} is a custom token inserted at the end of file inclusions,
> to guarantee that no file ends with a partially defined item.

### {{Comment}}

- `$(` 'Character'<sup>*</sup> `$)`

#### Behavior

- `$(` 'Character'<sup>*</sup> `$)`
    1. If 'Character'<sup>*</sup> contains a `$(`, raise <span style="color:#AB5753;">**Error 1: Nested comments are not supported**</span>.

### {{FileInclusion}}

- `$[` 'MathCharacter'<sup>+</sup> `$]`

#### Behavior

- `$[` 'MathCharacter'<sup>+</sup> `$]`
    1. Let {{Filename}} be 'MathCharacter'<sup>+</sup>
    1. If there is no file at {{Filename}}, raise <span style="color:#AB5753;">**Error 2: File does not exist**</span>
    1. If g{FilesIncluded} does not contain {{Filename}}:
        1. Add {{Filename}} to g{FilesIncluded}
        1. Insert the contents of the file and a custom {`EOF`} token at the current point

## Statements

### {{Statement}}

- {{Comment}}
- {{Scope}}
- {{VariableDeclaration}}
- {{DVCondition}}
- {{FHypothesis}}
- {{Assertion}}

> **Note**:
>
> A {{Statement}} is an {{Item}} that can be inside a {{Scope}}, unlike what metamath.pdf says.

#### Behavior

These aren't even options, but these errors may exist for increased user-friendliness.
- {{ConstantDeclaration}}
    1. Raise <span style="color:#AB5753;">**Error 3: Constant declarations must be global**</span>
- {{FileInclusion}}
    1. Raise <span style="color:#AB5753;">**Error 4: File inclusions must be global**</span>
- {`EOF`}
    1. Raise <span style="color:#AB5753;">**Error 5: File did not end in global scope**</span>

### {{Scope}}

- `${` {{Statement}}<sup>*</sup> `$}`

#### Behavior

- `${` {{Statement}}<sup>*</sup> `$}`
    1. When the `${` token is parsed, push a new scope to g{Scopes}
    1. When the `$}` token is parsed, call (pop g{Scopes}):whenDelete

### {{ConstantDeclaration}}

- `$c` {MathSymbol}<sup>+</sup> `$.`

#### Behavior

- `$c` {MathSymbol}<sup>+</sup> `$.`
    1. Assert: g{Scopes}:length = 1
    1. For each let _symbol_ : {MathSymbol}:
        1. If g{Scopes}:last:activeVariables contains _symbol_, raise <span style="color:#AB5753;">**Error 6: Variables may not be redeclared as constants**</span>
        1. If g{InactiveVariables} contains _symbol_, raise <span style="color:#AB5753;">**Error 6: Variables (even when out of scope) may not be redeclared as constants**</span>
        1. If g{Constants} contains _symbol_, raise <span style="color:#AB5753;">**Error 7: Constant symbols may not be redeclared**</span>
        1. Add _symbol_ to g{Constants}
        1. Add _symbol_ to g{MathSymbols}

### {{VariableDeclaration}}

- `$v` {MathSymbol}<sup>+</sup> `$.`

#### Behavior

- `$v` {MathSymbol}<sup>+</sup> `$.`
    1. Assert: g{Scopes} is nonempty
    1. For each let _symbol_ : {MathSymbol}:
        1. If g{Constants} contains _symbol_, raise <span style="color:#AB5753;">**Error 8: Constant symbols may not be redeclared as variables**</span>
        1. If g{Scopes}:last:activeVariables contains _symbol_, raise <span style="color:#AB5753;">**Error 9: Variables may not be redeclared (in scope)**</span>
        1. Add _symbol_ to g{Scopes}:last:activeVariables
        1. Add _symbol_ to g{MathSymbols}

### {{DVCondition}}

- `$d` {MathSymbol}<sup>2+</sup> `$.`

#### Behavior

- `$d` {MathSymbol}<sup>2+</sup> `$.`
    1. Assert: g{Scopes} is nonempty
    1. For each let _variable_ : {MathSymbol}
        1. If g{Scopes}:last:activeVariables does Not contain _variable_, raise <span style="color:#AB5753;">**Error 11: Variable is not declared (in scope)**</span>
    1. Add **this** to g{Scopes}:last:DVConditions

### {{FHypothesis}}

- {Label} `$f` {MathSymbol} {MathSymbol} `$.`

#### :typecode and :variable

- {Label} `$f` {MathSymbol} {MathSymbol} `$.`
    1. Let :typecode and :variable be the {MathSymbol} and {MathSymbol}, respectively.

#### Behavior

- {Label} `$f` {MathSymbol} {MathSymbol} `$.`
    1. Assert: g{Scopes} is nonempty
    1. If g{Constants} does Not contain :typecode, raise <span style="color:#AB5753;">**Error 10: Type of variable is not a declared constant**</span>
    1. If g{Scopes}:last:activeVariables does Not contain :variable, raise <span style="color:#AB5753;">**Error 11: Variable is not declared (in scope)**</span>
    1. If g{Scopes}:last:FHypotheses has an {{FHypothesis}} whose :variable = this:**variable**, raise <span style="color:#AB5753;">**Error 12: $f statement when a previous $f statement for this variable is also active**</span>
    1. If g{FHypotheses} has an {{FHypothesis}} whose :variable = this:**variable**, and whose :typecode ≠ **this**:typecode, raise <span style="color:#AB5753;">**Error 13: This $f chooses a different type for this variable than a previous $f**</span>
    1. Add **this** to g{Scopes}:last:fHypotheses
    1. Add **this** to g{FHypotheses}
    1. Call g:addLabel({Label}, **this**)

### {{EssentialHypothesis}}

- {Label} `$e` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`

#### :typecode and :expression

- {Label} `$e` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`
    1. Let :typecode be the {MathSymbol}
    1. Let :expression be the {MathSymbol}<sup>*</sup>

#### Behavior

- {Label} `$e` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`
    1. Assert: g{Scopes} is nonempty
    1. If g{Constants} does Not contain :typecode, raise <span style="color:#AB5753;">**Error 14: Type of expression is not a declared constant**</span>
    1. For each let _symbol_ : :expression:
        1. If g{Scopes}:last:activeMathSymbols does Not contain :typecode, raise <span style="color:#AB5753;">**Error 15: Symbol is not currently declared**</span>
        1. Call g:checkPossVarHasF(_symbol_)
    1. Add **this** to g{Scopes}:last:essentialHypotheses
    1. Call g:addLabel({Label}, **this**)

### {{Assertion}}

- {Label} `$a` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`
- {Label} `$p` {MathSymbol} {MathSymbol}<sup>*</sup> `$=` {{ProofDetails}} `$.`

#### :typecode and :expression

- {Label} `$a` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`
- {Label} `$p` {MathSymbol} {MathSymbol}<sup>*</sup> `$=` {{ProofDetails}} `$.`
    1. Let :typecode be the {MathSymbol}
    1. Let :expression be the {MathSymbol}<sup>*</sup>

#### :mandatoryVariables

- {Label} `$a` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`
- {Label} `$p` {MathSymbol} {MathSymbol}<sup>*</sup> `$=` {{ProofDetails}} `$.`
    1. Assert: g{Scopes} is nonempty
    1. Let _mandatoryVariables_ be :expression:variables
    1. For each let _hypothesis_ : g{Scopes}:last:activeEssentialHypotheses:
        1. _mandatoryVariables_ ++= _hypothesis_:expression:variables

#### :mandatoryHypotheses

- {Label} `$a` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`
- {Label} `$p` {MathSymbol} {MathSymbol}<sup>*</sup> `$=` {{ProofDetails}} `$.`
    1. Assert: g{Scopes} is nonempty
    1. Let _mandatoryHypotheses_ be the empty list
    1. For each let _fHypothesis_ : g{Scopes}:last:activeFHypotheses:
        1. If :mandatoryVariables has _fHypothesis_:variable:
            1. Append _fHypothesis_ to _mandatoryHypotheses_
    1. _mandatoryHypotheses_ ++= g{Scopes}:last:activeEssentialHypotheses

#### :mandatoryDVPairs

- {Label} `$a` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`
- {Label} `$p` {MathSymbol} {MathSymbol}<sup>*</sup> `$=` {{ProofDetails}} `$.`
    1. Assert: g{Scopes} is nonempty
    1. Let _pairs_ be a new set
    1. For each let (_first_, _second_) in g{Scopes}:last:activeDVPairs:
        1. If :mandatoryVariables has both _first_ and _second_, add (_first_, _second_) to _pairs_
    1. Return _pairs_

#### Behavior

- {Label} `$a` {MathSymbol} {MathSymbol}<sup>*</sup> `$.`
- {Label} `$p` {MathSymbol} {MathSymbol}<sup>*</sup> `$=` {{ProofDetails}} `$.`
    1. Assert: g{Scopes} is nonempty
    1. If g{Constants} does Not contain :typecode, raise <span style="color:#AB5753;">**Error 14: Type of expression is not a declared constant**</span>
    1. For each let _symbol_ : :expression:
        1. If g{Scopes}:last:activeMathSymbols does Not contain :typecode, raise <span style="color:#AB5753;">**Error 15: Symbol is not currently declared**</span>
        1. Call g:checkPossVarHasF(_symbol_)
    1. Call g:addLabel({Label}, **this**)

> **Note**:
>
> More code is run at {{ProofDetails}}

### {{ProofDetails}}

- {Label}<sup>+</sup>
- `(` {Label}<sup>*</sup> `)` {CompressedProofNumber}<sup>+</sup>

#### Behavior

- ({Label} | `?`)<sup>+</sup>
    1. Let _proof stack_
    1. For each let _label_ : {Label}<sup>+</sup>
        1. If _label_ is `?`, append a filler statement to the _proof stack_.
        1. Else if g:label(_label_) is None, raise <span style="color:#AB5753;">**Error 24: Unrecognized label**</span>
        1. Else if g:label(_label_) is a hypothesis, append it to the _proof stack_
        1. Else, let _assertion_ be g:label(_label_).
            1. Let _number of hypotheses_ be _assertion_:hypotheses:length
            1. Take _number of hypotheses_ elements from the _proof stack_ and assign them to _assertion_:hypotheses (reversing the order if necessary such that the first hypothesis is assigned the bottommost entry), and (uniquely) unify along all the assignments.
            1. If there are not enough elements in the stack, raise <span style="color:#AB5753;">**Error 18: Step uses more hypotheses than proven**</span>
            1. If a unification is not possible, raise <span style="color:#AB5753;">**Error 17: Cannot unify steps in compressed proof**</span>
            1. If two :mandatoryVariables of the assertion are replaced with expressions _A_ and _B_, and there is a corresponding :mandatoryDVPairs, then:
                1. If _A_ and _B_ have variables in common, raise <span style="color:#AB5753;">**Error 21: Disjoint variable condition violated**</span>
                1. If _A_ X. _B_ is not a subset of the :parentNode:mandatoryDVPairs, raise <span style="color:#AB5753;">**Error 22: Disjoint variable condition not satisfied (Hint: add $d ...)**</span>
    1. If there is more than one element in the _proof stack_, raise <span style="color:#AB5753;">**Error 19: More than one statement remaining at end of proof**</span>
    1. Assert: There is only one element in the _proof stack_
    1. If _proof stack_\[0] does not match ":parentNode:typecode :parentNode:expression", raise <span style="color:#AB5753;">**Error 20: A statement different from stated was proven**</span>

- `(` {Label}<sup>*</sup> `)` {CompressedProofNumber}<sup>+</sup>
    1. Let _proof stack_
    1. Let _reference stack_ be a copy of :parentNode:mandatoryHypotheses
    1. For each let _label_ : {Label}<sup>+</sup>
        1. If g:label(_label_) is None, raise <span style="color:#AB5753;">**Error 24: Unrecognized label**</span>
        1. Let Some(_node_) be g:label(_label_)
        1. Append _node_ to the _reference stack_
    1. **Note**: At this point, the behavior for {CompressedProofNumber} is run
    1. If there is more than one element in the _proof stack_, raise <span style="color:#AB5753;">**Error 19: More than one statement remaining at end of proof**</span>
    1. Assert: There is only one element in the _proof stack_
    1. If _proof stack_\[0] does not match ":parentNode:typecode :parentNode:expression", raise <span style="color:#AB5753;">**Error 20: A statement different from stated was proven**</span>

> **Note**:
>
> In the case of completed proofs, unification is easy, as all variables have a
> corresponding floating hypothesis that must be proven. As such, it is trivial
> to determine, for each variable, its corresponding substitution in the proof.
>
> The more general unification algorithm is of course much more complicated.

### {CompressedProofNumber}

- `?`
- `Z`
- \[`U`-`Y`]<sup>*</sup> {Whitespace}<sup>?</sup> \[`A`-`T`]<sup>+</sup>

#### Behavior

- `?`
    1. Append a filler statement to the _proof stack_.
- `Z`
    1. If the _proof stack_ is empty, raise <span style="color:#AB5753;">**Error 16: No subproof to recall**</span>
    1. Duplicate the top of the _proof stack_ and append it to the _reference stack_.
- \[`U`-`Y`]<sup>*</sup> {Whitespace}<sup>?</sup> \[`A`-`T`]<sup>+</sup>
    1. Let _base5_ be \[`U`-`Y`]<sup>*</sup> parsed as base 5
    1. Let _base20_ be \[`A`-`T`]<sup>+</sup> parsed as base 20
    1. Let _index_ be 20 * (1 + _base5_) + _base20_
    1. If _reference stack_\[_index_] does not exist, raise <span style="color:#AB5753;">**Error 23: Proof failed: Compressed number too large and does not reference anything**</span>
    1. If _reference stack_\[_index_] is a subproof or hypothesis, append it to the _proof stack_
    1. Else, let _assertion_ be _reference stack_\[_index_].
        1. Let _number of hypotheses_ be _assertion_:hypotheses:length
        1. Take _number of hypotheses_ elements from the _proof stack_ and assign them to _assertion_:hypotheses (reversing the order if necessary such that the first hypothesis is assigned the bottommost entry), and (uniquely) unify along all the assignments.
        1. If there are not enough elements in the stack, raise <span style="color:#AB5753;">**Error 18: Step uses more hypotheses than proven**</span>
        1. If a unification is not possible, raise <span style="color:#AB5753;">**Error 17: Cannot unify steps in compressed proof**</span>
        1. If two :mandatoryVariables of the assertion are replaced with expressions _A_ and _B_, and there is a corresponding :mandatoryDVPairs, then:
            1. If _A_ and _B_ have variables in common, raise <span style="color:#AB5753;">**Error 21: Disjoint variable condition violated**</span>
            1. If _A_ X. _B_ is not a subset of the :parentNode:mandatoryDVPairs, raise <span style="color:#AB5753;">**Error 22: Disjoint variable condition not satisfied (Hint: add $d ...)**</span>

## Tokens and characters

### 'Character'

- 'WhitespaceCharacter'
- 'MathCharacter'
- `$`

### {Whitespace}

- 'WhitespaceCharacter'<sup>+</sup>

### 'WhitespaceCharacter'

- Space ` `
- Tab `\t`
- Carriage return `\r`
- Line feed `\n`
- Form feed `\f` (`\x0C` if `\f` not supported)

### {Label}

- {LabelCharacter}<sup>+</sup>

### 'LabelCharacter'

- \[`a`-`z`]
- \[`A`-`Z`]
- \[`0`-`9`]
- one of `- _ .`

### {MathSymbol}

- 'MathCharacter'<sup>+</sup>

### 'MathCharacter'

- 'LabelCharacter'
- one of ``! " # % & ' ( ) * + , / : ; < = > ? @ [ \ ] ^ ` { | } ~``

##### Every vector space has a basis

1. Using the Axiom of Choice, there is a choice function from nonzero singleton spans to nonzero vectors.
1. The range of that function is a basis.
