#let title = "The TRAX Standard - 1st Edition (DRAFT)"
#let mono = "Berkeley Mono Variable"

#show raw: set text(font: mono)

#let syntax(defs: []) = {
  for def in defs {
    [
      ======= #label(def.at(0))
      #text(font: mono, size: 9pt)[
        #def.at(0) ::= #def.at(1)
      ]

    ]
  }
}

#let messages(defs: []) = {
  let properties(label, a) = {
    let property(b) = {
      grid(
        columns: (auto, 4pt, 1fr),
        text(font: mono, size: 10pt, b.at(0)) + text(style: "italic", " [" + b.at(1) + "]") + " :",
        "",
        b.at(2),
      )
    }

    if a.len() != 0 {
      [
        #text(weight: "bold", [#label properties:])
        #pad(left: 16pt, if type(a.at(0)) == str {
          property(a)
        } else {
          for prop in a {
            property(prop) + "\n"
          }
        })

      ]
    }
  }

  let example(e) = {
    text(weight: "bold", [Example:])
    pad(left: 16pt, raw(block: true, lang: "xml", e))
  }

  for def in defs {
    [
      === The #text(font: mono, size: 9pt, [#def.at(0)]) message
      #pad(left: 16pt, [
        #text(weight: "bold", [Action:])
        #pad(left: 16pt, def.at(1))

        #if def.len() == 3 {
          example(def.at(2))
        }

        #if def.len() == 4 {
          properties("Required", def.at(2))
          example(def.at(3))
        }

        #if def.len() == 5 {
          properties("Required", def.at(2))
          properties("Optional", def.at(3))
          example(def.at(4))
        }
      ])
    ]
  }
}

#show link: underline

#align(center, text(17pt)[
  #title
])

#align(center, [
  *Carter Reeb* \
  Santa Fe College \
  Gainesville, FL \
  _#link("mailto:carter.reeb@sfcollege.edu")_
])

#set page(paper: "us-letter", header: grid(columns: (1fr, 1fr), align(left)[
  #locate(loc => [
    #numbering("1", loc.page() - 1)
  ])
], align(right)[
  #title
]))

#set heading(numbering: "1.1")
#show heading: it => {
  if it.level == 7 {
    "<" + counter(heading).display((..nums) => nums.pos().at(6)) + ">  "
  } else {
    let number = if it.numbering != none {
      counter(heading).display(it.numbering)
      h(16pt, weak: true)
    }

    pad(y: 16pt, number + it.body)
  }
}

#pagebreak()

#show outline.entry.where(level: 1): it => {
  v(12pt, weak: true)
  strong(it)
}

#show outline.entry: it => {
  if it.body.has("children") {
    it
  }
}

#outline(depth: 5, indent: true, fill: pad(left: 16pt, repeat(".")))

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
      "  ':' | [A-Z] | '_' | [a-z] | [#xC0-#xD6] | [#xD8-#xF6] | [#xF8-#x2FF] | [#x370-#x37D] | [#x37F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] | [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] | [#x10000-#xEFFFF]",
    ),
    (
      "NameChar",
      [#link("NameStartChar") #"| '-' | '.' | [0-9] | #xB7 | [#x0300-#x036F] | [#x203F-#x2040]"],
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
      (link("Name") + " " + link("Eq") + " " + link("PropValue  ")),
    ),
    ("Modifier", (link("Name"))),
    ("ETag", ("'</' " + link("Name") + " " + link("S") + "? '>'")),
    (
      "content",
      (
        link("CharData") + "? ((" + link("element") + " | " + link("Reference") + " | " + link("Comment") + ") " + link("CharData") + "?)*"
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

= Client-Server Model
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
#messages(
  defs: (
    (
      "get",
      "Request to load a document",
      (),
      ("doc", "URL", "Path of the document to load"),
      "<get doc=\"todo.trax\" />",
    ),
    ("redirect", "Change the URL", ("url", "URL", "The new URL"), (
      "cosmetic",
      "modifier",
      "Only updates the URL visually without loading anything",
    ), "<redirect url=\"trax.tcp:qoogle.com/search.trax\" cosmetic />"),
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
          "integer",
          "If neither the start nor end modifiers are provided, replaces the nth child of target. If start is provided, insert at the nth position. If end is provided, insert at the nth-to-last position.",
        ),
      ),
      "<insert target='document.trax/Section?title=\"Giant Rat\"' start index=\"1\">\n\t<Text style=\"italic\">\n\t\tMove over rats, I'm the new 2nd child haha jaja\n\t</Text>\n</insert>\n\n/* No contents means the target element is deleted */\n<insert target=\"todo.trax/Frame/Body/Todo#1\" />\n\n/* Loads a new document */\n<insert target=\"document.trax\">\n\t<document>\n\t\t<H1> welcome to my website </H1>\n\t\t<Link dest=\"trax.tcp:coolmathgames.com\"> Leave this godforsaken place </Link>\n\t</document>\n</insert>",
    ),
    (
      "insertProp",
      "Insert or replace a property/modifier in the target element. If the key property isn't present, replaces the value of the property at the given position. If the value property isn't present, inserts a modifier if either the start/end modifiers are present, else replaces the key of the property at the given position. If neither are present, deletes the property at the given position. If no props are passed, deletes all properties.",
      ("target", "URL", "The path of the element to target"),
      (
        ("key", "string", "The attribute's key"),
        ("value", "any", "The attribute's value"),
        ("start", "modifier", "Places the property/modifier at the start",),
        ("end", "modifier", "Places the property/modifier at the end",),
        (
          "index",
          "integer",
          "If neither the start nor end modifiers are provided, replaces the nth property/modifier of target. If start is provided, insert at the nth position. If end is provided, insert at the nth-to-last position.",
        ),
      ),
      "<insertProp target=\"todo.trax/Frame/Body/Todo#1\" key=\"done\" end />",
    ),
  ),
)

#bibliography("bib.yml")