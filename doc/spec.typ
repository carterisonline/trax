#let title = "The TRAX Standard - 1st Edition (DRAFT)"
#let mono = "Berkeley Mono Variable"

#set page(paper: "us-letter")

#show raw: set text(font: mono)

#let em = 11pt
#let box = em * 1.5

#let spread3(f) = {
  let ret(a) = f(a.at(0), a.at(1), a.at(2));
  return ret
}

#let bold(..a) = text(..a, weight: "bold")
#let italic(..a) = text(..a, style: "italic")
#let lpad(..a) = pad(..a, left: box)

#let xml(..a) = raw(..a, lang: "xml")
#let xmlBlock(..a) = xml(..a, block: true)

#let field(ident, value) = {
  grid(columns: (auto, box / 4, 1fr), ident + " :", "", value)
}

#let typedField(ident, type, value) = field(raw(ident) + italic("[" + type + "]"), value)

#let fields(fn, label, props) = {
  if props.len() != 0 {
    [
      #bold(label)
      #lpad(if type(props.at(0)) == str {
        fn(props)
      } else {
        for prop in props {
          fn(prop) + "\n"
        }
      })
    ]
  }
}

#let exceptions(defs) = {
  let ctx(props) = fields(spread3(typedField), "Context", props)
  for def in defs {
    [
      == #raw(def.at(0))
      #lpad([
        #bold("Condition:")
        #lpad(def.at(1))

        #if def.len() == 3 {
          ctx(def.at(2))
        }
      ])
    ]
  }
}

#let elements(defs: []) = {
  let properties(label, props) = fields(spread3(typedField), label + " properties", props)

  let required(p) = properties("Required", p)
  let optional(p) = properties("Optional", p)

  let example(e) = {
    bold("Example:")
    lpad(xmlBlock(e))
  }

  for def in defs {
    [
      === The #raw(def.at(0)) element
      #lpad([
        #if def.len() == 7 {
          [*Evaluation* [#raw(def.at(2))]:]
          lpad(def.at(1))

          if type(def.at(3)) != str {
            [*Side Effects*:]
            lpad(def.at(3))
          }
          required(def.at(4))
          optional(def.at(5))
          example(def.at(6))
        } else {
          [*Side Effects*:]
          lpad(def.at(1))
          required(def.at(2))
          optional(def.at(3))
          example(def.at(4))
        }
      ])
    ]
  }
}

#let syntax(defs: []) = {
  for def in defs {
    [
      ======= #label(def.at(0))
      #text(font: mono, size: .75em)[
        #def.at(0) ::= #def.at(1)
      ]

    ]
  }
}

#show link: underline

#align(center, text(1.75em)[
  #title
])

#align(center, [
  *Carter Reeb* \
  Santa Fe College \
  Gainesville, FL \
  _#link("mailto: carter.reeb@sfcollege.edu")_
])

#set page(header: grid(columns: (1fr, 1fr), align(left)[
  #locate(loc => [
    #numbering("1", loc.page() - 1)
  ])
], align(right)[
  #title
]))

#set heading(numbering: "1.1")
#show heading: it => {
  if it.level == 7 {
    "<" + str(counter(heading).display((..nums) => nums.pos().at(6))) + "> "
  } else {
    let number = if it.numbering != none {
      counter(heading).display(it.numbering)
      h(box, weak: true)
    }

    pad(y: box, number + it.body)
  }
}

#pagebreak()

#show outline.entry.where(level: 1): it => {
  v(box * 3 / 4, weak: true)
  strong(it)
}

#show outline.entry: it => {
  if it.body.has("children") {
    it
  }
}

#outline(depth: 5, indent: true, fill: lpad(repeat(".")))

#pagebreak()

= TRAX Markup <markup>
This specification defines an abstract language for describing documents and
applications, and some APIs for interacting with in-memory representations of
resources that use this language.

Both the document structure and communications layer utilize markup, a task
which requires the markup to be stable and extensible. TRAX utilizes the
foundations of XML -- a proven format for effectively representing both
structured data and user interfaces.

== Differences from standard XML
TRAX implements an XML 1.0 @xml parser with the following modifications:
- Adds modifiers - properties without set values. Equivalent to boolean attributes
  in HTML. Can be prefixed.
- Removes Processing Instructions @xml[Chapter 2.6 - Processing Instructions]
- Removes CDATA Sections @xml[Chapter 2.7 - CDATA Sections]
- Removes Document Type Declarations @xml[Chapter 2.8 - Prolog and Document Type
  Declaration]
- Removes Entity References/Definitions @xml[Chapter 4.1 - Character and Entity
  References] @xml[Chapter 4.2 - Entity Declarations]
- Changes comment syntax to C-style block comments, beginning with `/*` and ending
  with `*/`. Comments can span multiple lines and are not nested.

== Syntax

The TRAX markup syntax is defined below as a PEG @peg compatible formal grammar.

#syntax(
  defs: (
    ("document", [element Misc\*]),
    (
      "Char",
      "#x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]",
    ),
    ("S", "(#x20 | #x9 | #xD | #xA)+"),
    (
      "NameStartChar",
      "':' | [A-Z] | '_' | [a-z] | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]",
    ),
    (
      "NameChar",
      [#link("NameStartChar") #" | '-' | '.' | [0-9] | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]"],
    ),
    ("Name", [#link("NameStartChar") (#link("NameChar"))\*]),
    ("Names", [#link("Name") (\#x20 #link("Name"))\*]),
    ("Nmtoken", [(#link("NameChar"))+]),
    ("Nmtokens", [#link("Nmtoken") (\#x20 #link("Nmtoken"))\*]),
    ("CharData", "[^<]*"),
    (
      "Comment",
      [#"'/*' (("#link("Char") #"- '*') | ('*' ("#link("Char") #"- '/')) ('/' ("#link("Char") -#"'*')))* '*/'"],
    ),
    ("Misc", (link("Comment") + " | " + link("S"))),
    (
      "Element",
      (
        link("EmptyElemTag") + " | " + link("STag") + " " + link("content") + " " + link("ETag")
      ),
    ),
    (
      "STag",
      (
        "'<' " + link("Name") + " (" + link("S") + " " + link("Property") + ")* " + link("S") + "? '>'"
      ),
    ),
    (
      "Property",
      (link("Name") + " " + link("Eq") + " " + link("PropValue")),
    ),
    ("Modifier", (link("Name"))),
    ("ETag", ("'</' " + link("Name") + " " + link("S") + "? '>'")),
    (
      "content",
      (
        link("CharData") + "? ((" + link("element") + " | " + link("Reference") + " | " + link("Comment") + ") " + link("CharData") + "?) * "
      ),
    ),
  ),
)

= TRAX Uniform Resource Locators <turl>
Much like URLs in HTML @html, TRAX URLs are used for locating documents and
files on websites. In addition, TRAX URLs are used for locating elements, much
like XPath @xpath or CSS Selectors @csssel.

== Differences from HTML URLs
- The scheme component is optional, defaulting to the `file` scheme.
- The double-slash (`//`) preceding the authority component is optional.
- Question marks (`?`) are valid query delimiters.
- Query and fragment components can follow any path segments
- Query values cannot contain a forward slash (`/`)

== Examples of valid TRAX URLs
+ `trax.quic:website.com`
+ `hello.png`
+ `todo.trax/Frame/Body/Todo#1/H1`
+ `trax.tcp:airports.info/db.trax/Airport?iata=DAB/Runway?Obstruction.obstacle=road`

= Exceptions

#exceptions(
  (
    (
      "AsEvalModifierTypeMismatch",
      [Using the or `asEval` prefix on a modifier, where the prefix's operation
      returned a non-boolean.],
      (
        ("modifier", "str", "The name of the modifier"),
        ("url", "URL", "The URL queried"),
        ("value", "any", "The returned value"),
      ),
    ),
    ("DivideByZero", [Attempting to divide by zero]),
    ("DocumentParseException", "Parsing an invalid document", (
      ("location", "range|undefined", "The location of the problem"),
      ("description", "str", "A description of the problem"),
    )),
  ),
)

= Language Constructs
This section defines abstractions that affect runtime behavior as opposed to
grammar.

== Types
- `any`: Any type.
- `undefined`: The absence of a value where one was expected
- `void`: The expected absence of a value.
- `bool`: A boolean. Represented as the existence of a modifier in an element.
- `str`: A string/text.
- `int`: A signed 64-bit integer.
- `float`: An `IEEE 754-2019` -compliant floating point number. @ieee754
- `range`: A non-inclusive range of integers
- `url`: A string with the contents of a valid URL.
- `element`: An element.
- `attribute`: An attribute.
- `modifier`: A modifier.

== Referencing vs. Evaluation
When a process gets its value via. referencing, it will use the structural or
textual content of its target. For example, referencing an arithmetic element
will return the element itself as defined in the document.

When a process gets its value via. evaluation, it will use the return value of
its target. Evaluating a literal will use the literal itself as the value. For
example, evaluating an arithmetic "sum" element will return the sum of that
element's children. Evaluating an element with no evaluation process (e.g. view
or metadata elements) will return `void`.

If either of these processes fail, they will return `undefined`.

== The `asRef` prefix
Prefixing an attribute:
+ Requires that the attribute's value be a URL
+ Returns the referenced value at the URL when evaluated

Prefixing a modifier:
+ Is parsed as an attribute, with the value being a URL
+ When evaluated, will return `true` if the URL is valid

== The `asEval` prefix
Prefixing an attribute:
+ Requires that the attribute's value be a URL
+ Returns the evaluated value at the URL when evaluated.

Prefixing a modifier:
+ Is parsed as an attribute, with the value being a URL
+ Returns the evaluated value at the URL when evaluated. Raises
  `AsEvalModifierTypeMismatch` when a non-boolean is returned.

= Foundational Elements
Foundational Elements in TRAX can be used in both network and document contexts.

The evaluated values of specific children are labeled as `"#x"`, where `x` is
the 0-indexed child. Referenced values are the same, but are instead prefixed
with `@`.

== Arithmetic Elements
Each arithmetic element will cast integers into floating point values when other
`floats` are used in their evaluation process. Downcasting from `float` to `int`
is prohibited.

#elements(defs: ((
  "add",
  "Evaluates all children, and returns the sum of the values.",
  "int|float",
  "",
  (),
  (),
  "/* Evals to 6 */\n<add>\n\t1\n\t2\n\t<abs> -3 </abs>\n</add>",
), (
  "sub",
  "Returns #0 minus #1.",
  "int|float",
  "",
  (),
  (),
  "<sub> 1 2 </sub> /* Evals to -1 */",
), (
  "mul",
  "Evaluates all children, and returns the product of the values.",
  "int|float",
  "",
  (),
  (),
  "/* Evals to 360 */\n<mul>\n\t3\n\t<abs> -4 </abs>\n\t5\n\t6\n</mul>",
), (
  "div",
  "Returns #0 divided by #1",
  "int|float",
  [Raises `DivideByZero` when dividing by zero.],
  (),
  (),
  "<div> 1 4 </div> /* Evals to 0.25 */",
), (
  "mod",
  "Returns #0 modulo #1",
  "int|float",
  [Raises `DivideByZero` when \#1 = 0.],
  (),
  (),
  "<mod> 35 6 </mod> /* Evals to 5 */",
), (
  "abs",
  "Returns the absolute value of #0",
  "int|float",
  "",
  (),
  (),
  "<abs> -3 </abs> /* Evals to 3 */ ",
)))

= Network Elements
Unlike HTML @html, TRAX does not define its own network protocol, and instead
communicates using TRAX markup [@markup] over existing network-layer protocols
such as TCP, or cross-layer userspace protocols such as QUIC @quic.

By using this method, TRAX benefits:

+ Only one parser besides TRAX URLs [@turl] is required to implement the entire
  standard
  - Any optimization provides benefits to all of TRAX's subsystems
  - Less work for project mantainers
+ Network messages can be constructed within Documents, eliminating the need for
  specialized APIs
+ Multiple network messages can be sent in a single request
+ All communication can be streamed from a single persistent two-way socket

== Message Directives
#elements(
  defs: (
    (
      "get",
      "Request to load a document",
      (),
      ("doc", "URL", "Path of the document to load"),
      "<getdoc = \"todo.trax\" />",
    ),
    (
      "redirect",
      "Change the URL",
      (
        ("connection", "URL", "The current connection"),
        ("url", "URL", "The new URL"),
      ),
      (
        "cosmetic",
        "modifier",
        "Only updates the URL visually without loading anything",
      ),
      "<redirect connection=\"trax.tcp:qoogle.com#0\" url=\"trax.tcp:qoogle.com/search.trax\" cosmetic />",
    ),
    (
      "insert",
      "Insert the contents of this message somewhere in the current document. Replaces the target if neither the start, end nor index properties are provided. ",
      ("target", "URL", "The path of the element to target"),
      (
        (
          "start",
          "modifier",
          "Places the contents as a child of target, at the start",
        ),
        (
          "end",
          "modifier",
          "Places the contents as a child of target, at the end",
        ),
        (
          "index",
          "int",
          "If neither the start nor end modifiers are provided, replaces the nth child of target. If start is provided, insert at the nth position. If end is provided, insert at the nth-to-last position.",
        ),
      ),
      "<insert target='document.trax#0/Section?title=\"Giant Rat\"' start index=\"1\">\n\t<Text style=\"italic\">\n\t\tMove over rats, I'm the new 2nd child haha jaja\n\t</Text>\n</insert>\n\n/* No contents means the target element is deleted */\n<insert target=\"todo.trax#0/Frame/Body/Todo#1\" />\n\n/* Loads a new document */\n<insert target=\"document.trax#0\">\n\t<document>\n\t\t<H1> welcome to my website </H1>\n\t\t<Link dest=\"trax.tcp:coolmathgames.com\"> Leave this godforsaken place </Link>\n\t</document>\n</insert>",
    ),
    (
      "insertProp",
      "Insert or replace a property/modifier in the target element. If the key property isn't present, replaces the value of the property at the given position. If the value property isn't present, inserts a modifier if either the start/end modifiers are present, else replaces the key of the property at the given position. If neither are present, deletes the property at the given position. If no props are passed, deletes all properties.",
      ("target", "URL", "The path of the element to target"),
      (
        ("key", "str", "The attribute's key"),
        ("value", "any", "The attribute's value"),
        ("start", "modifier", "Places the property/modifier at the start",),
        ("end", "modifier", "Places the property/modifier at the end",),
        (
          "index",
          "int",
          "If neither the start nor end modifiers are provided, replaces the nth property/modifier of target. If start is provided, insert at the nth position. If end is provided, insert at the nth-to-last position.",
        ),
      ),
      "<insertProp target=\"todo.trax#0/Frame/Body/Todo#1\" key=\"done\" end />",
    ),
  ),
)

#pagebreak()
== An Example of Network Communication
#show raw: set text(size: 1.1em)
#set text(size: 0.75em)
#table(
  columns: (box, 1.2fr, 1fr, 1fr),
  inset: 5pt,
  align: horizon,
  [#text(size: em, "#")],
  [#text(size: em, [*Server Response*])],
  [#text(size: em, [*Client Request*])],
  [#text(size: em, [*Description*])],
  [1],
  [],
  xml("<get />"),
  [Client attempts to load `trax.tcp:example.com`],
  [2],
  xml("<redirect connection=\"trax.tcp:example.com#0\" url=\"todo.trax\" />"),
  [],
  [Server redirects the client to the initial document located at
  `trax.tcp:example.com/todo.trax`.

  No other connections from our IP to the initial address are open, so the server
  identifies us as connection \#0.],
  [3],
  [],
  xml("<get doc=\"todo.trax\" />"),
  [Client requests initial document after it was redirected],
  [4],
  xml(
    "<insert target=\"todo.trax#0\">\n\t<document>\n\t\t...\n\t</document>\n</insert>",
  ),
  [],
  [Server tells us to replace the current tab's contents with the contents of
  `todo.trax`.

  It also identifies us as connection \#0 here because we're requesting a
  different address.],
  [5],
  xml(
    "<insert target=\"todo.trax#0/Frame/Body\" start>\n\t<Todo title=\"Do Laundry\" done />\n\t<Todo title=\"Work on TRAX\" desc=\"gonna take a while\" />\n</insert>",
  ),
  [],
  [The server tells connection \#0 (our only tab) to insert these two `Todo`s in
  our `Body` element.],
  [6],
  [],
  xml(
    "<insertProp\n\ttarget=\"todo.trax#0/Frame/Body/Todo#1\"\n\tkey=\"done\" end />",
  ),
  [The client has done something (user checked a box, etc.) that added the `done`
  modifier to the 2nd `Todo` element. The client notifies the server that it did
  so.],
  [7],
  xml("<insert path=\"todo.trax#0/Frame/Body/Todo#1\" />"),
  [],
  [In response, the server requests that connection \#0 (still us) inserts nothing
  over the 2nd `Todo` element, which deletes it.],
)

#set text(size: 11pt)
#show raw: set text(size: 8.8pt)

#bibliography("bib.yml")