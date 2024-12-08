# From Snake to Snek: a JIT compiler for snake game actions

Convert snake game motions into snek code! Lines of generated code are compiled and run, and the result of running your program is compared to the goal value. If they match, you win!

<img width="617" alt="image" src="https://github.com/user-attachments/assets/76cebd4d-acb5-4a16-8e39-7d98e0d7c71d">

## Running the game

Install the rust development environment from [here](https://www.rust-lang.org/tools/install) (if you do not have one).

In the project directory, run

```
> cargo run
```

## Game Controls & Rules

- Use the arrow keys on the keyboard to move the green snake.
- Eat the food to make the snake stronger (or longer).
- When the snake hits the border or itself, it dies.
- Each food item eaten corresponds to a token appended to the current program line.
    - +, -, * => binary operations
    - add1, sub1 => unary operations
    - END => end program line
    - def => saves result of running current line as a heap-allocated variable accessible in future prog lines
    - let => let binding for stack-allocated bindings
    - var => beginning of var definition in let binding
    - set => set value of variable declared in let binding
    - id => when eaten, all stored variable identifiers that can be used in the prog appear as blocks
    - int & end_int => number of spaces traveled between eating both blocks is the int appended to the prog line
    - (, ), {, }, |, ;, :=  => additional syntax
- Once a line ends, the line will be compiled and the result displayed on the right column of the game display
- All variables (heap- and stack-allocated) are displayed on the left column of the game display
- If the snake dies, the game is reset and all generated program lines and heap-allocated variables are lost

## Modified snek grammar
- expr_body -> let { var_binding* } { expr } | set identifier := expr ;
- var_binding -> var identifier := expr |
- expr -> term | expr_body
- term -> addend | term add_op addend
- addend -> factor | addend * factor
- factor -> summand | ( un_op summand )
- summand -> int | identifier | ( expr )
- add_op -> + | -
- un_op -> add1 | sub1


## Credits
Snake game starter code from [rust-snake](https://github.com/SLMT/rust-snake/tree/master)

Compiler base from our submission for 17-363 [forest flame](https://github.com/sherylm77/forest-flame?tab=readme-ov-file)
