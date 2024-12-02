# From snake to snek: a snake game interpreter

Snake game starter code from https://github.com/SLMT/rust-snake/tree/master

Compiler base from our submission for 17-363 forest flame: https://github.com/sherylm77/forest-flame?tab=readme-ov-file

Updates and todos in https://docs.google.com/document/d/104AJC2acIl6G65bTNkNUMvxEPmX92B7VDgXLwaj2TzM/edit?tab=t.0




## How To Run ?

First, install the Rust development evnironment from [here](https://www.rust-lang.org/tools/install) (if you do not have one).

Second, run the following command in the project directory:

```
> cargo run
```

Enjoy!

## Game Controls & Rules

- Use the arrow keys on the keyboard to move the green snake.
- Eat the food to make the snake stronger (or longer).
- When the snake hits the border or itself, it dies.
- The number of squares the snake moves is the number appended to the currently generated instruction line
- Each color of food represents a certain instruction
    - yellow: +
    - purple: -
    - blue: *
    - green: end of line
    - red: def (the result of the line generated will be stored as a heap-allocated variable)
- Once a line ends, the line will be compiled and the result displayed