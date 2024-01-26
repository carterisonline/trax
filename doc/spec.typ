#let title = "The TRAX Standard - 1st Edition (DRAFT)"

#let syntax(defs: []) = {
  for def in defs {
    [
      === #label(def.at(0))
      // copies made with the berkeley font are illegal but I like looking at it
      // will ship with a more boring libre font once publicly available
      #text(font: "Berkeley Mono Variable", size: 9pt)[
        #def.at(0) ::= #def.at(1)
      ]

    ]
  }
}

#show link: underline

#align(center, text(17pt)[
  #title
])

#align(
  center,
  [
    *Carter Reeb* \
    Santa Fe College \
    Gainesville, FL \
    _#link("mailto:carter.reev@go.sfcollege.edu")_ // please ask sf to fix this it's pissing me off that this is my real email
  ],
)

#set page(paper: "us-letter", header: grid(columns: (1fr, 1fr), align(left)[
  #locate(loc => [
    #numbering("1", loc.page() - 1)
  ])
], align(right)[
  #title
]))

#set heading(numbering: "1.1")
#show heading: it => {
  if it.level == 3 {
    "<" + counter(heading).display((..nums) => nums.pos().at(2)) + ">  "
  } else {
    let number = if it.numbering != none {
      counter(heading).display(it.numbering)
      h(16pt, weak: true)
    }

    pad(y: 16pt, number + it.body)
  }
}

#pagebreak()

= TRAX Markup
Both the document structure and communications layer utilize markup, a task
which requires the markup to be stable and extensible. TRAX utilizes the
foundations of XML -- a proven format for effectively representing both
structured data and user interfaces.

== Differences from standard XML
TRAX implements an XML 1.0 @xml parser with the following modifications:
- Adds modifiers - element attributes without set values. Equivalent to boolean
  attributes in HTML. Can be prefixed.
- Removes Processing Instructions @xml[Chapter 2.6 - Processing Instructions]
- Removes CDATA Sections @xml[Chapter 2.7 - CDATA Sections]
- Removes Document Type Declarations @xml[Chapter 2.8 - Prolog and Document Type
  Declaration]
- Removes Entity References/Definitions @xml[Chapter 4.1 - Character and Entity
  References] @xml[Chapter 4.2 - Entity Declarations]
- Changes comment syntax to C-style block comments, beginning with `/*` and ending
  with `*/`. Comments can span multiple lines and are *not* nested.

== Syntax

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
        "'<' " + link("Name") + " (" + link("S") + " " + link("Attribute") + ")* " + link("S") + "? '>'"
      ),
    ),
    (
      "Attribute",
      (link("Name") + " " + link("Eq") + " " + link("AttValue  ")),
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

#bibliography("bib.yml")