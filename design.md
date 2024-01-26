# Design

## Generational Application State
The only thing you need to communicate with a server - in practice - is a slightly modified XML parser. \
Message sent from the server are written to a "scratchpad" document (addressed as SCRATCH) on the client, and are executed from start to end when the server sends the `<commit />` tag.

Example diagram written from perspective of the client:

| Stage | RX                                                                                        | TX                                                           | Comment                                                                                                                                       |
| ----- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- |
| 1     | -                                                                                         | `<get doc="todo.trax" />`                                    | Client requests initial document (see [todo.trax](doc/todo.trax))                                                                             |
| 2     | `<load> (contents of todo.trax) </load>`                                                  | -                                                            | Server sends file contents                                                                                                                    |
| 3     | `<insert path="todo.trax/Frame/Body"> (contents of todo_todos.trax) </insert> <commit />` | -                                                            | Server sends a command to insert elements into the Body. For this example it's the initial todos (see [todo_todos.trax](doc/todo_todos.trax)) |
| 4     | -                                                                                         | `<add:prop path="todo.trax/Frame/Body/Todo%1" key="done" />` | Client checks the "done" checkbox, which pushes a diff to the server                                                                          |
| 5     | `<clear:elem path="todo.trax/Frame/Body/Todo%1" /> <commit />`                            | -                                                            | Server sends a command to clear (delete) the 2nd Todo element                                                                                 |