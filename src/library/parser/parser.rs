#![allow(dead_code)]

use library::doc::DocType::*;
use library::lexeme::definition::TokenType::*;
use library::lexeme::definition::{TokenKind, TokenType};
use library::lexeme::token::Token;
use library::parser::helper::*;
use library::parser::rust_type::*;

#[derive(Debug)]
struct SymbolTable {
    symbol_type: TokenType,
    symbol_modifier: Modifier,
    id_name: String,
    is_assigned: bool,
    is_ptr: bool,
    assigned_val: String,
}

#[derive(Debug)]
struct CStructMem {
    name: String,
    member_type: TokenType,
    identifier: String,
}

impl Clone for SymbolTable {
    fn clone(&self) -> SymbolTable {
        let id = self.id_name.clone();
        let val = self.assigned_val.clone();
        SymbolTable {
            assigned_val: val,
            id_name: id,
            symbol_modifier: self.symbol_modifier,
            symbol_type: self.symbol_type,
            is_ptr: self.is_ptr,
            is_assigned: self.is_assigned,
        }
    }
}

impl Clone for CStructMem {
    fn clone(&self) -> CStructMem {
        CStructMem {
            name: self.name.clone(),
            member_type: self.member_type,
            identifier: self.identifier.clone(),
        }
    }
}

struct Parser {
    from: usize,
    //for symbol table
    once_warned: bool,
    //default false
    in_block_stmnt: bool,
    //default false
    in_expr: bool,
    //default false
    in_switch: bool,
    //defalt false
    strict: bool,
    //default true
    in_main: bool,
    sym_tab: Vec<SymbolTable>,
    // structure book keeping
    struct_mem: Vec<CStructMem>,
    typde_def_table: Vec<String>,
    struct_in_body_declaration: bool,
}

pub fn init_parser(lexeme: &Vec<Token>, strict_parser: bool) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
    stream.push(CRUST.get_doc().to_string());

    let mut parser = Parser {
        from: 0,
        once_warned: false,
        in_block_stmnt: false,
        in_expr: false,
        in_switch: false,
        strict: strict_parser,
        in_main: false,
        sym_tab: Vec::new(),
        struct_mem: Vec::new(),
        typde_def_table: Vec::new(),
        struct_in_body_declaration: false,
    };
    stream.append(&mut parser.parse_program(&lexeme));
    stream
}

impl Parser {
    fn parse_program(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut lookahead: usize;
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lexeme.len() {
            lookahead = head;
            //match over token kind and token type
            match lexeme[head].get_type() {
                (TokenKind::Typedef, Typedef) => {
                    //typedef STRUCT struct_t or
                    //typedef struct typename {struct_def} new_type_name;
                    if lexeme[head + 1].get_token_type() == KeywordStruct {
                        //read in struct definition
                        while lexeme[head].get_token_type() != RightCurlyBrace {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }
                    }
                    while lexeme[head].get_token_type() != Semicolon {
                        temp_lexeme.push(lexeme[head].clone());
                        head += 1;
                    }
                    temp_lexeme.push(lexeme[head].clone());
                    stream.append(&mut self.parse_typdef(&temp_lexeme));
                    head += 1;
                    temp_lexeme.clear();
                }
                // matches any datatype
                (TokenKind::DataTypes, _) => {
                    //if token is modifiers , move lookahead pointer to next lexeme
                    if lexeme[head].get_token_type() == Signed
                        || lexeme[head].get_token_type() == Unsigned
                    {
                        lookahead += 1;
                    }
                    //To see whats after the given identifier
                    //ex : int a = 0; int a;
                    //     int foo(){}
                    lookahead += 2;
                    match lexeme[lookahead].get_token_type() {
                        // function declaration
                        LeftBracket => {
                            //inside the function
                            self.in_block_stmnt = true;
                            //move till end of function argument declaration
                            while lexeme[lookahead].get_token_type() != RightBracket {
                                lookahead += 1;
                            }
                            //move ahead of )
                            lookahead += 1;

                            // skip function body declaration
                            if lexeme[lookahead].get_token_type() != LeftCurlyBrace {
                                lookahead += 1;
                                head = lookahead;
                                //FIXME : Why is continue here ?
                                //continue; ??
                            }
                            // advance lookahead to end of block
                            lookahead = skip_block(&lexeme, lookahead + 1);
                            // collect entire function block
                            while head < lookahead {
                                let l: Token = lexeme[head].clone();
                                temp_lexeme.push(l);
                                head += 1;
                            }

                            //parse function block
                            stream.append(&mut self.parse_function(&temp_lexeme));
                            temp_lexeme.clear();

                            self.in_block_stmnt = false;
                        }

                        //array declaration found
                        LeftSquareBracket => {
                            lookahead = skip_stmt(&lexeme, lookahead);

                            // collect variable declaration
                            while head != lookahead {
                                let l: Token = lexeme[head].clone();
                                temp_lexeme.push(l);
                                head += 1;
                            }
                            // parse declaration
                            stream.append(&mut self.parse_array_declaration(&temp_lexeme));
                            temp_lexeme.clear();
                        }

                        // variable declaration or declaration + assignment
                        Semicolon | Comma | Assignment => {
                            lookahead = skip_stmt(&lexeme, lookahead);

                            // collect variable declaration
                            while head != lookahead {
                                let l: Token = lexeme[head].clone();
                                temp_lexeme.push(l);
                                head += 1;
                            }
                            // parse declaration
                            stream.append(&mut self.parse_declaration(&temp_lexeme, false));
                            temp_lexeme.clear();
                        }
                        Identifier => {
                            //in case of pointer declaration : int *a;
                            while lexeme[head].get_token_type() != Semicolon {
                                temp_lexeme.push(lexeme[head].clone());
                                head += 1;
                            }
                            temp_lexeme.push(lexeme[head].clone());
                            stream.append(&mut self.parse_declaration(&temp_lexeme, false));
                            head += 1;
                            temp_lexeme.clear();
                        }

                        _ => {}
                    };
                }

                // matches if statement
                (TokenKind::Keyword, KeywordIf) => {
                    // let mut temp_lexeme: Vec<Token> = Vec::new();

                    // move lookahead past conditon
                    while lexeme[lookahead].get_token_type() != RightBracket {
                        lookahead += 1;
                    }
                    lookahead += 1;

                    // move lookahead past block
                    if lexeme[lookahead].get_token_type() == LeftCurlyBrace {
                        lookahead = skip_block(&lexeme, lookahead + 1);
                    }
                    // move lookahead past block for 'if' without braces
                    else {
                        lookahead = skip_stmt(&lexeme, lookahead);
                    }
                    // collect if block
                    while head < lookahead {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }

                    // parse if
                    stream.append(&mut self.parse_if(&temp_lexeme));
                    temp_lexeme.clear();
                }

                (TokenKind::Keyword, KeywordElse) => {
                    stream.push("else".to_string());
                    head += 1;
                    lookahead = head;
                    if lexeme[head].get_token_type() == KeywordIf {
                        continue;
                    } else {
                        if lexeme[lookahead].get_token_type() == LeftCurlyBrace {
                            head += 1;
                            lookahead = skip_block(&lexeme, head) - 1;
                        } else {
                            lookahead = skip_stmt(&lexeme, lookahead);
                        }

                        while head < lookahead {
                            let l: Token = lexeme[head].clone();
                            temp_lexeme.push(l);
                            head += 1;
                        }
                        //** parse else body
                        stream.push("{".to_string());
                        stream.append(&mut self.parse_program(&temp_lexeme));
                        temp_lexeme.clear();
                        stream.push("}".to_string());
                    }
                }

                (TokenKind::Keyword, KeywordSwitch) => {
                    while lexeme[lookahead].get_token_type() != LeftCurlyBrace {
                        lookahead += 1;
                    }
                    lookahead += 1;

                    lookahead = skip_block(&lexeme, lookahead);
                    while head < lookahead {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }
                    self.in_switch = true;

                    stream.append(&mut self.parse_switch(&temp_lexeme));
                    temp_lexeme.clear();
                    self.in_switch = false;
                }

                (TokenKind::Keyword, KeywordWhile) => {
                    // let mut temp_lexeme: Vec<Token> = Vec::new();

                    // move lookahead past conditon
                    while lexeme[lookahead].get_token_type() != RightBracket {
                        lookahead += 1;
                    }
                    lookahead += 1;

                    // move lookahead past block
                    if lexeme[lookahead].get_token_type() == LeftCurlyBrace {
                        lookahead = skip_block(&lexeme, lookahead + 1);
                    }
                    // move lookahead past block for 'if' without braces
                    else {
                        lookahead = skip_stmt(&lexeme, lookahead);
                    }
                    // collect if block
                    while head < lookahead {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }

                    let was_in_switch: bool;
                    was_in_switch = self.in_switch;
                    self.in_switch = false;

                    // parse if
                    stream.append(&mut self.parse_while(&temp_lexeme));
                    self.in_switch = was_in_switch;
                    temp_lexeme.clear();
                }

                // matches do while statement
                (TokenKind::Keyword, KeywordDo) => {
                    // move lookahead past block
                    lookahead = skip_block(&lexeme, lookahead + 2);
                    lookahead = skip_stmt(&lexeme, lookahead);

                    // collect while block
                    while head < lookahead {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }
                    // parse while
                    let was_in_switch: bool;
                    was_in_switch = self.in_switch;
                    self.in_switch = false;

                    stream.append(&mut self.parse_dowhile(&temp_lexeme));
                    temp_lexeme.clear();

                    self.in_switch = was_in_switch;
                }

                // matches for statement
                (_, KeywordFor) => {
                    //read till end of for()
                    while lexeme[lookahead].get_token_type() != RightBracket {
                        lookahead += 1;
                    }
                    //forward the look ahead buffer if block statements found

                    lookahead += 1;
                    if lexeme[lookahead].get_token_type() == LeftCurlyBrace {
                        /*
                        for(x;y;z){
                            c=a+b;
                        }*/

                        lookahead = skip_block(&lexeme, lookahead);
                    } else {
                        /*
                        for(x;y;z)
                            c=a+b;
                            */
                        //read till end of following expression/statement. i.e semicolon
                        lookahead = skip_stmt(&lexeme, lookahead);
                    }

                    while head < lookahead {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }

                    let was_in_switch = self.in_switch;
                    self.in_switch = false;

                    stream.append(&mut self.parse_for(&temp_lexeme));
                    temp_lexeme.clear();
                    self.in_switch = was_in_switch;
                }

                // matches single and multi-line comment
                (TokenKind::Comments, _) => {
                    stream.push(lexeme[head].get_token_value() + "\n");
                    head += 1;
                }

                // assignment statements
                (_, Identifier) => {
                    // let mut temp_lexeme: Vec<Token> = Vec::new();
                    //identifier = expr
                    //identifier()
                    //identifier+expr
                    //identifier OP_INC|OP_DEC; =>postfix

                    match lexeme[head + 1].get_type() {
                        (TokenKind::Identifiers, Identifier) => {
                            if self
                                .typde_def_table
                                .contains(&lexeme[head].get_token_value())
                            {
                                lookahead = skip_stmt(&lexeme, lookahead);

                                // collect variable declaration
                                while head != lookahead {
                                    let l: Token = lexeme[head].clone();
                                    temp_lexeme.push(l);
                                    head += 1;
                                }
                                // parse declaration
                                stream.append(&mut self.parse_declaration(&temp_lexeme, false));
                            } else {
                                while lexeme[head].get_token_type() != Semicolon {
                                    temp_lexeme.push(lexeme[head].clone());
                                    head += 1;
                                }
                                temp_lexeme.push(lexeme[head].clone());
                                head += 1;
                                stream.append(&mut self.parse_class_decl(&temp_lexeme));
                            }
                            temp_lexeme.clear();
                        }

                        (TokenKind::AssignmentOperators, Assignment) => {
                            // move lookahead past statement
                            if lexeme[head + 3].get_token_type() == Comma {
                                lookahead = head + 3;
                                while head < lookahead + 1 {
                                    let l: Token = lexeme[head].clone();
                                    temp_lexeme.push(l);
                                    head += 1;
                                }
                                stream.append(&mut self.parse_assignment(&temp_lexeme));
                                temp_lexeme.clear();
                            } else {
                                lookahead = skip_stmt(&lexeme, lookahead);
                                // collect statement
                                while head < lookahead {
                                    let l: Token = lexeme[head].clone();
                                    temp_lexeme.push(l);
                                    head += 1;
                                }

                                // parse assignment
                                stream.append(&mut self.parse_assignment(&temp_lexeme));
                                temp_lexeme.clear();
                            }
                        }
                        (TokenKind::UnaryOperators, _) => {
                            if self.in_expr != true {
                                stream.push(lexeme[head].get_token_value());
                                stream.push(match lexeme[head + 1].get_token_type() {
                                    Increment => "+=1".to_string(),
                                    Decrement => "-=1".to_string(),
                                    _ => " ;".to_string(),
                                });
                                head += 2;
                            } else {
                                head += 2;
                            }
                        }
                        (TokenKind::BinaryOperators, _) => {
                            lookahead = skip_stmt(&lexeme, lookahead);

                            //check if overloaded operators is in effect like << >>
                            if lexeme[head + 2].get_token_type() == StringValue
                                || lexeme[head + 2].get_token_type() == CharValue
                            {
                                stream.push(
                                    "\n//This statement need to be handled manually \n".to_string(),
                                );
                                while head < lookahead {
                                    stream.push(lexeme[head].get_token_value());
                                    head += 1;
                                }
                            } else {
                                // move lookahead past statement
                                // collect statement
                                while head < lookahead {
                                    let l: Token = lexeme[head].clone();
                                    temp_lexeme.push(l);
                                    head += 1;
                                }

                                // parse assignment
                                stream.append(&mut self.parse_expr(&temp_lexeme));
                                temp_lexeme.clear();
                            }
                        }
                        (TokenKind::SpecialChars, LeftBracket) => {
                            //function call?
                            let mut temp_lexeme: Vec<Token> = Vec::new();
                            stream.push(lexeme[head].get_token_value());
                            stream.push(lexeme[head + 1].get_token_value());

                            head += 2;

                            let lookahead = skip_paranthised_block(&lexeme, head);
                            while head < lookahead - 1 {
                                temp_lexeme.push(lexeme[head].clone());
                                head += 1;
                            }
                            let mut parsed_expr = self.parse_params(&temp_lexeme);
                            stream.append(&mut parsed_expr);
                            stream.push(lexeme[head].get_token_value());
                            head += 1;
                        }
                        // (_, LEFT_SBRACKET) => {
                        //     while lexeme[head].get_token_type() != RIGHT_SBRACKET {
                        //         stream.push(lexeme[head].get_token_value());
                        //         head += 1;
                        //     }
                        //     stream.push(lexeme[head].get)
                        // }
                        (_, _) => {
                            if lexeme[head].get_token_type() != RightCurlyBrace {
                                stream.push(lexeme[head].get_token_value());
                            }
                            head += 1;
                        }
                    };
                }

                (TokenKind::UnaryOperators, _) => {
                    stream.push(lexeme[head + 1].get_token_value());
                    stream.push(match lexeme[head].get_token_type() {
                        Increment => "+=1".to_string(),
                        Decrement => "-=1".to_string(),
                        _ => " ;".to_string(),
                    });
                    head += 2;
                }

                (_, KeywordStruct) => {
                    if lexeme[head + 2].get_token_type() == LeftCurlyBrace {
                        //struct A{};
                        while lexeme[head].get_token_type() != RightCurlyBrace {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }
                        //push the right curly brace
                        temp_lexeme.push(lexeme[head].clone());
                        stream.append(&mut self.parse_struct(&temp_lexeme));
                        temp_lexeme.clear();
                        head += 2; //skip semicolon
                    } else {
                        //struct variable declaration

                        while lexeme[head].get_token_type() != Semicolon {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }
                        temp_lexeme.push(lexeme[head].clone());
                        head += 1;
                        stream.append(&mut self.parse_struct_decl(&temp_lexeme));
                        temp_lexeme.clear();
                    }
                }

                (_, KeywordUnion) => {
                    if lexeme[head + 2].get_token_type() == LeftCurlyBrace {
                        stream.push(UNION.get_doc().to_string());
                        //struct A{};
                        while lexeme[head].get_token_type() != RightCurlyBrace {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }
                        //push the right curly brace
                        temp_lexeme.push(lexeme[head].clone());
                        stream.append(&mut self.parse_union(&temp_lexeme));
                        temp_lexeme.clear();
                        head += 2; //skip semicolon
                    } else {
                        //struct variable declaration

                        while lexeme[head].get_token_type() != Semicolon {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }
                        temp_lexeme.push(lexeme[head].clone());
                        head += 1;
                        stream.append(&mut self.parse_union_decl(&temp_lexeme));
                        temp_lexeme.clear();
                    }
                }

                (_, KeywordClass) => {
                    if lexeme[head + 2].get_token_type() == LeftCurlyBrace {
                        //struct A{};
                        while lexeme[head].get_token_type() != RightCurlyBrace
                            || lexeme[head + 1].get_token_type() != Semicolon
                        {
                            temp_lexeme.push(lexeme[head].clone());
                            head += 1;
                        }

                        //push the right curly brace
                        temp_lexeme.push(lexeme[head].clone());
                        stream.append(&mut self.parse_class(&temp_lexeme));
                        temp_lexeme.clear();
                        head += 2; //skip semicolon
                    }
                }

                (_, KeywordEnum) => {
                    while lexeme[head].get_token_type() != Semicolon {
                        stream.push(lexeme[head].get_token_value());
                        head += 1;
                    }
                    head += 1;
                }
                (_, KeywordReturn) => {
                    let mut t = head;
                    stream.push(NO_RETURN.get_doc().to_string());

                    while lexeme[t].get_token_type() != Semicolon {
                        t += 1;
                    }

                    if t != lexeme.len() - 1 {
                        while lexeme[head].get_token_type() != Semicolon {
                            stream.push(lexeme[head].get_token_value());
                            head += 1;
                        }
                        stream.push(lexeme[head].get_token_value());
                        head += 1;
                    } else {
                        //convert to shorthand notation
                        head += 1;
                        if self.in_main {
                            stream.push("std::process::exit(".to_string());
                            while lexeme[head].get_token_type() != Semicolon {
                                stream.push(lexeme[head].get_token_value());
                                head += 1;
                            }
                            stream.push(");".to_string());
                        } else {
                            while lexeme[head].get_token_type() != Semicolon {
                                stream.push(lexeme[head].get_token_value());
                                head += 1;
                            }
                        }
                        head += 1;
                    }
                }
                (TokenKind::Preprocessors, _) => {
                    let line_num = lexeme[head].get_token_line_num();
                    if self.once_warned == false {
                        stream.push(INCLUDE_STMT.get_doc().to_string());
                    } else {
                        //if warned once is set, we have already started the preprocessor block and commented out using multi line comment.
                        //which is ended at line L369 below. Need to pop that out before writing next pre-processor statement.
                        stream.pop();
                    }

                    //read in the current line as is.
                    while lexeme[head].get_token_line_num() != line_num {
                        stream.push(lexeme[head].get_token_value());
                        head += 1;
                    }
                    stream.push(lexeme[head].get_token_value() + "\n");
                    stream.push("**/\n".to_string());
                    head += 1;

                    self.once_warned = true;
                }
                (TokenKind::SpecialChars, LeftBracket) => {
                    let mut temp_lexeme: Vec<Token> = Vec::new();
                    while lexeme[head].get_token_type() != RightBracket {
                        temp_lexeme.push(lexeme[head].clone());
                        head += 1;
                    }
                    temp_lexeme.push(lexeme[head].clone());

                    stream.append(&mut self.parse_typecast(&temp_lexeme));
                    head += 1;
                }

                // if all fails
                (_, _) => {
                    if lexeme[head].get_token_type() != RightCurlyBrace {
                        if lexeme[head].get_token_type() == Comma {
                            stream.push(";".to_string());
                        } else if lexeme[head].get_token_type() == KeywordBreak {
                            if !self.in_switch {
                                stream.push(lexeme[head].get_token_value());
                            }
                        } else {
                            stream.push(lexeme[head].get_token_value());
                        }
                    }
                    head += 1;
                }
            };
        }
        //return the rust lexeme to main
        stream
    }

    /**
     * print_lexemes: DEBUG_ONLY
     * prints the lexemes in the lexeme vector
     * from index start to end
     */
    fn print_lexemes(lexeme: &Vec<Token>, start: usize, end: usize) {
        println!("----------lexeme-start------------");
        for i in start..end {
            println!(
                "Line Num {} , {}> {:?}",
                lexeme[i].get_token_line_num(),
                i,
                lexeme[i],
            );
        }
        println!("----------lexeme-end------------");
    }

    /**
     * parse_function:
     * parse c/c++ function into rust equivalent function
     */
    fn parse_function(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut temp_lexeme: Vec<Token> = Vec::new();
        let mut head: usize = 3;
        let mut lookahead: usize = head;
        let mut stream: Vec<String> = Vec::new();

        stream.push("fn".to_string());
        stream.push(lexeme[1].get_token_value());
        stream.push("(".to_string());

        // parse arguments differently for functions that are not main
        // since rust does not have arguments or return type for main
        if lexeme[1].get_token_type() != Main {
            // collect arguments
            while lexeme[lookahead].get_token_type() != RightBracket {
                lookahead += 1;
            }
            while head < lookahead {
                let l: Token = lexeme[head].clone();
                temp_lexeme.push(l);
                head += 1;
            }
            // parse arguments
            stream.append(&mut self.parse_arguments(&temp_lexeme));
            temp_lexeme.clear();

            stream.push(")".to_string());

            // parse return type
            if let Some(rust_type) = parse_type(lexeme[0].get_token_type(), Modifier::Default) {
                if rust_type != "void".to_string() {
                    stream.push("->".to_string());
                    stream.push(rust_type);
                }
            }

            stream.push("{".to_string());
        }
        // declare argc and argv inside main, if required
        else {
            //parsing main function
            self.in_main = true;
            stream.push(")".to_string());
            stream.push("{".to_string());
            if lexeme[head].get_token_type() != RightBracket {
                if self.strict == false {
                    stream.push(NO_STRICT.get_doc().to_string());
                    stream.push("let mut argv: Vec<_> = std::env::args().collect();".to_string());
                    stream.push("let mut argc = argv.len();".to_string());
                } else {
                    stream.push(STRICT.get_doc().to_string());

                    stream.push("let argv: Vec<_> = std::env::args().collect();".to_string());
                    stream.push("let argc = argv.len();".to_string());
                }
            }
        }

        while lexeme[head].get_token_type() != LeftCurlyBrace {
            head += 1
        }
        head += 1;

        // collect function body
        // len - 1  so that '}' is excluded
        while head < lexeme.len() - 1 {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse function body
        stream.append(&mut self.parse_program(&temp_lexeme));
        stream.push("}".to_string());
        self.in_main = false;
        stream
    }

    /**
     * parse-arguments:
     * parse c/c++ formal arguments in the function signature
     * into rust equivalent arguments
     */
    fn parse_arguments(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        while head < lexeme.len() {
            let mut declaration_lexeme: Vec<Token> = Vec::new();
            while head < lexeme.len() && lexeme[head].get_token_type() != Comma {
                //create a subset of argument declaration

                declaration_lexeme.push(lexeme[head].clone());

                head += 1;
            }
            head += 1;

            let mut parsed_argument = self.parse_declaration(&declaration_lexeme, true);

            stream.append(&mut parsed_argument);
            stream.push(",".to_string());
        }
        stream
    }

    fn parse_params(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();

        //create expression termination token ';'
        let terminal_token = Token::new(";".to_string(), TokenKind::SpecialChars, Semicolon, 0, 0);
        let mut head: usize = 0;
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lexeme.len() {
            if lexeme[head].get_token_type() == Comma {
                temp_lexeme.push(terminal_token.clone());

                let mut parsed_expr = self.parse_expr(&temp_lexeme);
                //pop the tailing semicolon
                parsed_expr.pop();
                stream.append(&mut parsed_expr);
                stream.push(",".to_string());
                temp_lexeme.clear();
            } else {
                temp_lexeme.push(lexeme[head].clone());
            }
            head += 1;
        }
        temp_lexeme.push(terminal_token.clone());
        let mut parsed_expr = self.parse_expr(&temp_lexeme);
        //pop the tailing semicolon
        parsed_expr.pop();
        stream.append(&mut parsed_expr);

        return stream;
    }
    /**
     * parse_declaration:
     * parse c/c++ declaration into rust
     * equivalent statements */
    fn parse_declaration(
        &mut self,
        lexeme: &Vec<Token>,
        argument_declaration: bool,
    ) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();

        //  let mut sym_tab: Vec<SymbolTable> = Vec::new();
        //self.sym_tab.clear();
        let mut sym: SymbolTable = SymbolTable {
            symbol_type: Others,
            symbol_modifier: Modifier::Default,
            id_name: "undefined_var".to_string(),
            is_assigned: false,
            is_ptr: false,
            assigned_val: "NONE".to_string(),
        };

        //check if there is any modifier present

        let (token_kind, token_type) = lexeme[0].get_type();

        let mut type_index = 0;
        if token_kind == TokenKind::Modifiers {
            //type name can be found in next lexeme
            type_index = 1;
            match token_type {
                Signed => {
                    sym.symbol_modifier = Modifier::Signed;
                }
                Unsigned => {
                    sym.symbol_modifier = Modifier::Unsigned;
                }
                KeywordStatic => {
                    sym.symbol_modifier = Modifier::Static;
                }
                KeywordConst => {
                    sym.symbol_modifier = Modifier::Const;
                }
                _ => {}
            }
        }

        let type_token = &lexeme[type_index];
        let typdef_type = type_token.get_token_value(); //get the type name

        let mut head: usize = type_index + 1;
        //let sym_idx:usize=0;
        while head < lexeme.len() {
            match lexeme[head].get_token_type() {
                Identifier => sym.id_name = lexeme[head].get_token_value(),

                Assignment => {
                    sym.is_assigned = true;
                    sym.assigned_val = "".to_string();
                    head += 1;
                    let mut br = 0;
                    let mut lhead = head;
                    if sym.is_ptr == true {
                        if lexeme[head].get_token_type() == Null {
                            while lexeme[lhead].get_token_type() != Semicolon
                                && lexeme[lhead].get_token_type() != Comma
                            {
                                lhead += 1;
                            }
                            sym.is_assigned = false;
                        }
                    }
                    let mut temp_lex: Vec<Token> = Vec::new();
                    while lexeme[head].get_token_type() != Semicolon
                        && !(br == 0 && lexeme[head].get_token_type() == Comma)
                    {
                        if lexeme[head].get_token_type() == LeftBracket {
                            br += 1;
                        }
                        if lexeme[head].get_token_type() == RightBracket {
                            br -= 1;
                        }
                        temp_lex.push(lexeme[head].clone());
                        //parse assigned value for expression

                        head += 1;
                    }
                    temp_lex.push(lexeme[head].clone());
                    let a_val = self.parse_expr(&temp_lex);
                    let mut a_value = String::new();
                    for val in a_val {
                        a_value = a_value + &val;
                    }
                    sym.assigned_val.push_str(a_value.as_str());

                    continue;
                }

                Semicolon | Comma => {
                    // used enum value in the symbol table
                    sym.symbol_type = type_token.get_token_type();
                    self.sym_tab.push(sym.clone());
                }
                //int * a ;
                Multiplication => {
                    sym.is_ptr = true;
                }
                _ => {
                    sym.assigned_val.push_str(&lexeme[head].get_token_value());
                }
            };
            head += 1;
        }

        if !self.struct_in_body_declaration {
            if self.strict == false {
                stream.push(NO_STRICT.get_doc().to_string());
            } else {
                stream.push(STRICT.get_doc().to_string());
            }
        }
        //from `from` start declaration statement generation
        let (_, sym_table_right) = self.sym_tab.split_at(self.from);
        for i in sym_table_right {
            // get identifier
            //for declaration out of any blocks(global)
            self.from += 1;
            match i.symbol_modifier {
                Modifier::Const => {
                    stream.push("const".to_string());
                }
                _ => {
                    if !self.struct_in_body_declaration {
                        if self.strict == false {
                            if self.in_block_stmnt == true {
                                stream.push("let mut".to_string());
                            } else {
                                stream.push("static mut".to_string());
                            }
                        } else {
                            if self.in_block_stmnt == true {
                                stream.push("let".to_string());
                            } else {
                                stream.push("static".to_string());
                            }
                        }
                    }
                }
            }
            stream.push(i.id_name.clone());
            stream.push(":".to_string());

            if i.is_ptr == true {
                stream.push("&".to_string());

                if self.strict == false {
                    stream.push("mut".to_string());
                }
            }
            // get the rust type
            if let Some(rust_type) = parse_type(i.symbol_type, i.symbol_modifier) {
                if rust_type == "_".to_string() {
                    //not able to find the type, let the rust compiler do the type inference.
                    stream.pop();
                } else {
                    stream.push(rust_type);
                }
            } else {
                // if type parser dint return Some type, then it must be user defined type.
                //TODO : should check the typedef table
                stream.push(typdef_type.clone());
            }

            // take care of assignment
            if i.is_assigned {
                stream.push("=".to_string());
                if i.is_ptr == true {
                    stream.push("&".to_string());
                }
                if self.strict == false && i.is_ptr == true {
                    stream.push("mut".to_string());
                }

                stream.push((&i.assigned_val).to_string());
            }
            if !argument_declaration {
                stream.push(";".to_string());
            }
        }
        stream
    }

    /* parse simple typedef definition of form
     * typedef typename newtype;
     */
    fn parse_typdef(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();

        let mut struct_token_index = 1;
        let mut alias_type_index = 2;
        //extract struct definition if present
        //typdef struct strcutname{} alias_type;
        let mut struct_lexeme: Vec<Token> = Vec::new();

        if lexeme.get(1).unwrap().get_token_type() == TokenType::KeywordStruct {
            let mut head: usize = 1;
            while lexeme.get(head).unwrap().get_token_type() != TokenType::RightCurlyBrace {
                struct_lexeme.push(lexeme.get(head).unwrap().clone());
                head += 1;
            }
            struct_lexeme.push(lexeme.get(head).unwrap().clone());
            //3 token will be struct name
            struct_token_index = 2;
            //move head to point to alias type token
            head += 1;
            alias_type_index = head;
        }

        let alias_type = lexeme[alias_type_index].get_token_value();
        let mut struct_token = lexeme[struct_token_index].clone();

        if alias_type == struct_token.get_token_value() {
            //rust doesnt allow to have struct with same name as type alias, so we need to scramble the name here
            let new_name = struct_token.get_token_value() + "_alas_t";
            struct_lexeme[1].set_token_value(&new_name);
            struct_token = struct_lexeme[1].clone();
        }

        stream.push("\n".to_string());
        let mut parsed_struct = self.parse_struct(&struct_lexeme);
        stream.append(&mut parsed_struct);

        stream.push("type".to_string());
        stream.push(alias_type.clone() + "=");
        self.typde_def_table.push(alias_type.clone());
        if let Some(typ) = parse_type(struct_token.get_token_type(), Modifier::Default) {
            stream.push(typ);
        } else {
            stream.push(struct_token.get_token_value());
        }
        stream.push(";".to_string());
        return stream;
    }

    /**
     * parse_if:
     * parse c/c++ if statements into rust
     * equivalent statements
     */
    fn parse_if(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;

        stream.push("if".to_string());
        stream.push("(".to_string());
        head += 1;

        //skip '('
        head += 1;

        // condition
        while lexeme[head].get_token_type() != RightBracket {
            stream.push(lexeme[head].get_token_value());
            head += 1;
        }
        head += 1;
        stream.push(")".to_string());
        stream.push("== true".to_string());
        stream.push("{".to_string());

        if lexeme[head].get_token_type() == LeftCurlyBrace {
            head += 1;
        }

        // collect if body
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lexeme.len() {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse if body
        stream.append(&mut self.parse_program(&temp_lexeme));

        stream.push("}".to_string());
        stream
    }

    /**
     * parse_while:
     * parse c/c++ while statements into rust
     * equivalent statements
     */
    fn parse_while(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut no_cond = false;
        head += 1;

        //skip '('
        head += 1;
        // condition
        let mut cond_stream: Vec<String> = Vec::new();
        while lexeme[head].get_token_type() != RightBracket {
            cond_stream.push(lexeme[head].get_token_value());
            head += 1;
        }
        if cond_stream.len() == 1
            && (cond_stream[0] == "1".to_string() || cond_stream[0] == "true".to_string())
        {
            no_cond = true;
        }
        head += 1;

        if lexeme[head].get_token_type() == LeftCurlyBrace {
            head += 1;
        }

        // collect while body
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lexeme.len() {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse while body
        let mut body_stream = &mut self.parse_program(&temp_lexeme);

        if no_cond == true {
            stream.push("loop".to_string());
        } else {
            stream.push("while".to_string());
            stream.push("(".to_string());
            stream.append(&mut cond_stream);
            stream.push(")".to_string());
            stream.push("== true".to_string());
        }
        stream.push("{".to_string());
        stream.append(&mut body_stream);

        stream.push("}".to_string());
        stream
    }

    /**
     * parse_dowhile:
     * parse c/c++ do while statements into rust
     * equivalent statements
     */
    fn parse_dowhile(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut temp_stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut lookahead: usize;

        head += 2;
        lookahead = head;

        lookahead = skip_block(&lexeme, lookahead) - 1;
        // collect while body
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while head < lookahead {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse while body

        temp_stream.append(&mut self.parse_program(&temp_lexeme));
        temp_lexeme.clear();

        head += 3;
        if (lexeme[head].get_token_value() == String::from("1")
            || lexeme[head].get_token_value() == String::from("true"))
            && lexeme[head + 1].get_token_type() == RightBracket
        {
            stream.push("loop".to_string());
            stream.push("{".to_string());
            stream.append(&mut temp_stream);

            stream.push("}".to_string());
        } else {
            stream.push("while".to_string());
            stream.push("{".to_string());
            stream.append(&mut temp_stream);
            stream.push("(".to_string());
            while lexeme[head].get_token_type() != RightBracket {
                stream.push(lexeme[head].get_token_value());
                head += 1;
            }
            stream.push(")".to_string());
            stream.push("== true".to_string());
            stream.push("}".to_string());
            stream.push("{".to_string());
            stream.push("}".to_string());
        }
        stream.push(";".to_string());
        stream
    }

    fn parse_switch(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut head: usize = 2;
        let mut lookahead: usize = 2;
        let mut stream: Vec<String> = Vec::new();
        let mut temp_lexeme: Vec<Token> = Vec::new();

        stream.push("match".to_string());

        // find starting of switch block
        while lexeme[lookahead].get_token_type() != LeftCurlyBrace {
            lookahead += 1;
        }
        // {
        // move back to find the variable/result to be matched
        lookahead -= 1;
        // single variable
        if lookahead - head == 1 {
            stream.push(lexeme[lookahead - 1].get_token_value());
        }
        // expression
        else {
            while head < lookahead {
                let l: Token = lexeme[head].clone();
                temp_lexeme.push(l);
                head += 1;
            }
            head -= 1;
            stream.append(&mut self.parse_program(&temp_lexeme));

            temp_lexeme.clear();
        }
        // move forward to the starting of the block
        head += 3;
        stream.push("{".to_string());

        //head is at case
        lookahead = skip_block(&lexeme, head) - 1;
        while head < lookahead {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        stream.append(&mut self.parse_case(&temp_lexeme));
        stream.push("}".to_string());
        stream
    }

    fn parse_case(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        //head is at case
        let mut head: usize = 0;
        let mut lookahead: usize;
        let mut temp_lexeme: Vec<Token> = Vec::new();
        let mut def: bool = false;
        //look whether default case is handled for exaustive search
        while head < lexeme.len() {
            if lexeme[head].get_token_type() == KeywordDefault {
                stream.push("_".to_string());
                def = true;
            } else {
                head += 1; //head is at matching value
                stream.push(lexeme[head].get_token_value());
            }

            head += 1; // head is at :
            stream.push("=>".to_string());

            // either brace or no brace
            head += 1;
            if lexeme[head].get_token_type() == LeftCurlyBrace {
                head += 1;
                lookahead = skip_block(&lexeme, head) - 1;
            } else {
                lookahead = head;
                while lookahead < lexeme.len()
                    && lexeme[lookahead].get_token_type() != KeywordCase
                    && lexeme[lookahead].get_token_type() != KeywordDefault
                {
                    lookahead += 1;
                }
            }
            while head < lookahead {
                let l: Token = lexeme[head].clone();
                temp_lexeme.push(l);
                head += 1;
            }
            stream.push("{".to_string());
            stream.append(&mut self.parse_program(&temp_lexeme));
            stream.push("}".to_string());

            if head < lexeme.len() && lexeme[head].get_token_type() == RightCurlyBrace {
                head += 1;
            }
            temp_lexeme.clear();
        }
        if def == false {
            stream.push("_".to_string());
            stream.push("=>".to_string());
            stream.push("{".to_string());
            stream.push("}".to_string());
        }
        stream
    }

    /**
     * parse_for:
     * parse c/c++ do while statements into rust
     * equivalent statements
     *
     * Identify infinite loops and replace for with loop{}
     */
    fn parse_for(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut lookahead: usize;
        let mut temp_lexeme: Vec<Token> = Vec::new();

        while lexeme[head].get_token_type() != LeftBracket {
            head += 1;
        }
        head += 1;
        lookahead = head;

        //for (int i =0; )
        let decl: bool = if lexeme[head].get_token_kind() == TokenKind::DataTypes {
            true
        } else {
            false
        };
        // let mut no_init:bool; //no initialization
        let mut no_cond: bool = false; //if no condition to terminate
        let mut no_updation: bool = false; //no inc/dec of loop counter

        let mut body: Vec<String> = Vec::new();
        let mut updation: Vec<String> = Vec::new();
        let mut term_cond: Vec<String> = Vec::new();
        // initial assignment
        lookahead = skip_stmt(&lexeme, lookahead);

        //incase of initialization expression for (;i<10;i++) ; common case
        if head + 1 < lookahead {
            while head < lookahead {
                let l: Token = lexeme[head].clone();
                temp_lexeme.push(l);
                head += 1;
            }

            if decl == true {
                stream.append(&mut self.parse_declaration(&temp_lexeme, false));
            } else {
                stream.append(&mut self.parse_assignment(&temp_lexeme));
            }
        } else {
            head += 1;
            // no_init = true;
        }
        temp_lexeme.clear();

        // terminating condition
        lookahead = skip_stmt(&lexeme, lookahead);

        if head + 1 < lookahead {
            while head < lookahead - 1 {
                term_cond.push(lexeme[head].get_token_value());
                head += 1;
            }
        } else {
            no_cond = true;
        }
        head += 1;
        temp_lexeme.clear();

        lookahead = head;
        // update expression
        while lexeme[lookahead].get_token_type() != RightBracket {
            let l: Token = lexeme[lookahead].clone();
            temp_lexeme.push(l);
            lookahead += 1;
        }
        //no_updation
        if head == lookahead {
            no_updation = true;
        } else {
            temp_lexeme.push(Token::new(
                String::from(";"),
                TokenKind::SpecialChars,
                Semicolon,
                0,
                0,
            ));
            updation.append(&mut self.parse_program(&temp_lexeme));
            temp_lexeme.clear();
        }
        head = lookahead;
        head += 1;
        if lexeme[head].get_token_type() == LeftCurlyBrace {
            head += 1;
            lookahead = skip_block(&lexeme, head);
        } else {
            lookahead = skip_stmt(&lexeme, head);
        }

        // lookahead = skip_block(&lexeme, lookahead);
        while head < lookahead {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        body.append(&mut self.parse_program(&temp_lexeme));

        if no_cond == true {
            stream.push("loop".to_string());
        } else {
            stream.push("while".to_string());
            stream.append(&mut term_cond); //append termianating condition
        }
        stream.push("{".to_string());
        stream.append(&mut body);
        if no_updation != true {
            stream.append(&mut updation);
        }

        stream.push("}".to_string());

        stream
    }

    /* parse_assignment:
     * parse c/c++ assignment statements into rust equivalent code
     * compound assignments must be converted to declarations
     * as rust doesnt support compound assignment
     */
    fn parse_assignment(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        // let mut lookahead = lexeme.len();
        let mut thead: usize = 2;
        let mut lexeme1: Vec<Token> = Vec::new();

        let mut n = 2;
        let m = 3;
        let mut tstream: Vec<String> = Vec::new();
        if lexeme[n].get_token_kind() == TokenKind::UnaryOperators {
            while lexeme[thead].get_token_type() != Semicolon {
                lexeme1.push(lexeme[thead].clone());
                thead += 1;
            }
            lexeme1.push(lexeme[thead].clone());
            stream.push(lexeme[0].get_token_value());
            stream.push(lexeme[1].get_token_value());
            stream.append(&mut self.parse_expr(&lexeme1));
        } else if lexeme[m].get_token_kind() == TokenKind::UnaryOperators {
            while lexeme[thead].get_token_type() != Semicolon {
                lexeme1.push(lexeme[thead].clone());
                thead += 1;
            }
            lexeme1.push(lexeme[thead].clone());
            stream.push(lexeme[0].get_token_value());
            stream.push(lexeme[1].get_token_value());
            stream.append(&mut self.parse_expr(&lexeme1));
        } else if lexeme[m].get_token_kind() == TokenKind::BinaryOperators {
            while lexeme[thead].get_token_type() != Semicolon {
                lexeme1.push(lexeme[thead].clone());
                thead += 1;
            }
            lexeme1.push(lexeme[thead].clone());
            stream.push(lexeme[0].get_token_value());
            stream.push(lexeme[1].get_token_value());
            stream.append(&mut self.parse_expr(&lexeme1));
        } else if lexeme[n].get_token_type() == BitwiseAnd {
            stream.push(lexeme[0].get_token_value());
            stream.push(lexeme[1].get_token_value());

            while lexeme[thead].get_token_type() != Semicolon {
                stream.push(lexeme[thead].get_token_value());
                thead += 1;
            }
        } else {
            if lexeme[m].get_token_type() == Assignment {
                while lexeme[thead].get_token_type() != Semicolon
                    && lexeme[thead].get_token_type() != Comma
                {
                    lexeme1.push(lexeme[thead].clone());
                    thead += 1;
                }
                lexeme1.push(lexeme[thead].clone());
                stream.append(&mut self.parse_program(&lexeme1));
            }
            stream.push(lexeme[0].get_token_value());
            stream.push(lexeme[1].get_token_value());
            if lexeme[n].get_token_kind() == TokenKind::UnaryOperators {
                stream.push(lexeme[m].get_token_value());
            } else {
                stream.push(lexeme[n].get_token_value());
                n += 1;
                if lexeme[n].get_token_type() == LeftBracket
                    || lexeme[n].get_token_type() == LeftSquareBracket
                {
                    while lexeme[n].get_token_type() != Semicolon {
                        stream.push(lexeme[n].get_token_value());
                        n += 1;
                    }
                }
                stream.push(";".to_string());
            }
        }
        if tstream.len() > 0 {
            stream.append(&mut tstream);
        }
        stream
    }

    /* parse_expr:
     * parse c/c++ expression statements into rust equivalent code
     */
    fn parse_expr(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        // let mut lookahead = lexeme.len();
        let mut tstream: Vec<String> = Vec::new();
        let mut thead: usize = 0;

        let mut prev_id = " ".to_string();
        let mut typ = Others;
        //a=b+c++;
        while thead < lexeme.len() && lexeme[thead].get_token_type() != Semicolon {
            if lexeme[thead].get_token_kind() == TokenKind::UnaryOperators {
                if lexeme[thead].get_token_type() == SizeOf {
                    stream.push("std::mem::size_of(".to_string());
                    thead += 2;
                    if lexeme[thead].get_token_kind() == TokenKind::DataTypes {
                        if let Some(t) =
                            parse_type(lexeme[thead].get_token_type(), Modifier::Default)
                        {
                            stream.push(t)
                        }
                    } else {
                        stream.push(lexeme[thead].get_token_value());
                    }
                    stream.push(")".to_string());
                    thead += 1;
                } else {
                    //incase of post
                    if typ == Identifier {
                        tstream.push(prev_id.clone());
                        tstream.push(match lexeme[thead].get_token_type() {
                            Increment => "+=1".to_string(),
                            Decrement => "-=1".to_string(),
                            _ => " ;".to_string(),
                        });
                        tstream.push(";".to_string());

                        thead += 1;
                    //continue;
                    }
                    // incase of pre
                    else {
                        stream.push("(".to_string());
                        stream.push(lexeme[thead + 1].get_token_value());
                        stream.push(match lexeme[thead].get_token_type() {
                            Increment => "+=1".to_string(),
                            Decrement => "-=1".to_string(),
                            _ => " ;".to_string(),
                        });
                        stream.push(")".to_string());
                        thead += 1;
                    }
                }
            } else if lexeme[thead].get_token_kind() == TokenKind::SpecialChars {
                if lexeme[thead].get_token_type() == LeftBracket
                    && lexeme[thead + 1].get_token_kind() == TokenKind::DataTypes
                {
                    //type cast expression.
                    let mut temp_lexeme: Vec<Token> = Vec::new();
                    while thead < lexeme.len()
                        && lexeme[thead].get_token_type() != Semicolon
                        && lexeme[thead].get_token_type() != Comma
                    {
                        temp_lexeme.push(lexeme[thead].clone());
                        thead += 1;
                    }
                    let mut parsed_stmnt = self.parse_typecast(&temp_lexeme);
                    stream.append(&mut parsed_stmnt);
                    //move back from end of statement, so next move thead inc will not panic
                    thead -= 1;
                }
            } else {
                stream.push(lexeme[thead].get_token_value());
            }

            typ = lexeme[thead].get_token_type();
            prev_id = lexeme[thead].get_token_value();

            thead += 1;
        }
        stream.push(";".to_string());
        if tstream.len() > 0 {
            stream.append(&mut tstream);
        }
        stream
    }

    /**
     * Parse simple type case statement of form (int)a or (int *)a
     * */
    fn parse_typecast(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();

        //get the type

        let mut lookahead: usize = 0;

        while lexeme[lookahead].get_token_type() != RightBracket {
            lookahead += 1;
        }

        //lookahead - 1 is *, then its a pointer
        let is_ptr = if lexeme[lookahead - 1].get_token_type() == Multiplication {
            true
        } else {
            false
        };
        let type_index = lookahead - 2;
        //move lookahead to poit to expression
        lookahead += 1;
        while lookahead < lexeme.len() {
            //TODO: skip address of op operator ?

            stream.push(lexeme[lookahead].get_token_value());
            if lexeme[lookahead].get_token_type() == BitwiseAnd && !self.strict {
                stream.push(" mut ".to_string());
            }
            lookahead += 1;
        }

        //get the rust equivalent type
        let mut typ: String = "".to_string();
        if let Some(t) = parse_type(lexeme[type_index].get_token_type(), Modifier::Default) {
            typ = t;
        }

        if is_ptr {
            //TODO : Equivalent of this??
            if self.strict {
                stream.push(" as & ".to_string());
            } else {
                stream.push(" as &mut ".to_string());
            }
            stream.push(typ);
        } else {
            stream.push(" as ".to_string());
            stream.push(typ);
        }

        //check if its a pointer
        return stream;
    }

    fn parse_array_declaration(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut typ: String = " ".to_string();

        //int a[10];
        if let Some(t) = parse_type(lexeme[0].get_token_type(), Modifier::Default) {
            typ = t;
        }
        if !self.struct_in_body_declaration {
            if self.strict == true {
                stream.push(STRICT.get_doc().to_string());
                stream.push("let".to_string());
            } else {
                stream.push(NO_STRICT.get_doc().to_string());
                stream.push("let mut".to_string());
            }
        }

        let mut head = 0;
        stream.push(lexeme[head + 1].get_token_value());
        stream.push(":".to_string());
        stream
            .push("[".to_string() + &typ[..] + ";" + &lexeme[head + 3].get_token_value()[..] + "]");
        head = 5;
        let mut lookahead = head;
        while lexeme[lookahead].get_token_type() != Semicolon {
            lookahead += 1;
        }
        let mut temp_lexeme: Vec<Token> = Vec::new();
        if lexeme[head].get_token_type() == Comma {
            temp_lexeme.push(lexeme[0].clone());
            //move to next
            head += 1;
            while lexeme[head].get_token_type() != Semicolon {
                temp_lexeme.push(lexeme[head].clone());
                head += 1;
            }
            stream.push(";".to_string());
            temp_lexeme.push(lexeme[head].clone());
            stream.append(&mut self.parse_program(&temp_lexeme));
        } else if lexeme[head].get_token_type() == Assignment {
            while lexeme[head].get_token_type() != Semicolon
                && lexeme[head].get_token_type() != RightCurlyBrace
            {
                stream.push(match lexeme[head].get_token_type() {
                    LeftCurlyBrace => "[".to_string(),

                    _ => lexeme[head].get_token_value(),
                });
                head += 1;
            }
            stream.push("]".to_string());
            stream.push(";".to_string());
        } else {
            stream.push(";".to_string());
        }

        stream
    }

    // not tested
    fn parse_struct(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        stream.push(lexeme[head].get_token_value()); //push the keyword struct
        head += 1;
        //push the struct id_name
        stream.push(lexeme[head].get_token_value()); //push the struct name
        let name = lexeme[head].get_token_value();
        stream.push("{".to_string());
        head += 2;
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while lexeme[head].get_token_type() != RightCurlyBrace {
            while lexeme[head].get_token_type() != Semicolon {
                temp_lexeme.push(lexeme[head].clone());
                head += 1
            }
            temp_lexeme.push(lexeme[head].clone());
            head += 1;
            //check if line comment is present
            if lexeme[head].get_token_type() == SingleLineComment {
                temp_lexeme.push(lexeme[head].clone());
                head += 1;
            }
            stream.append(&mut self.parse_struct_inbody_decl(&temp_lexeme, &name));
            temp_lexeme.clear();
        }
        stream.push(lexeme[head].get_token_value() + "\n");

        stream
    }

    // not tested
    fn parse_struct_inbody_decl(&mut self, lexeme: &Vec<Token>, _name: &String) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        //push the identifier
        self.struct_in_body_declaration = true;
        let mut head: usize = 0;
        let mut array_decl = false;
        while head + 2 < lexeme.len() {
            if lexeme[head].get_token_type() == LeftSquareBracket
                && lexeme[head + 2].get_token_type() == RightSquareBracket
                && lexeme[head + 1].get_token_type() == NumberInteger
            //this pretty much defines the array decl, i guess
            {
                array_decl = true;
            };
            head += 1;
        }

        let mut parsed_decl: Vec<String>;
        if array_decl {
            parsed_decl = self.parse_array_declaration(&lexeme);
        } else {
            parsed_decl = self.parse_declaration(&lexeme, true);
        }
        //remove tailing semicolon added by parser
        if parsed_decl.last().unwrap() == ";" {
            parsed_decl.pop();
        }
        self.struct_in_body_declaration = false;
        stream.append(&mut parsed_decl);
        stream.push(",".to_string());
        stream
    }

    // not tested
    fn parse_struct_decl(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();

        stream.push(STRUCT_INIT.get_doc().to_string());
        stream.push("let".to_string());
        let mut head = 1;
        //struct FilePointer fp;
        let struct_name = lexeme[head].get_token_value();
        head += 1;
        stream.push(lexeme[head].get_token_value()); //push the identifer => let a
        stream.push("=".to_string());
        stream.push(struct_name.clone());
        stream.push("{".to_string());

        for row in &self.struct_mem {
            if row.name == struct_name {
                stream.push(row.identifier.clone());
                stream.push(":".to_string());
                stream.push(get_default_value_for(row.member_type));
                stream.push(",".to_string());
            }
        }
        stream.push("};".to_string());

        stream
    }

    //parse tagged union
    fn parse_union(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        stream.push("enum".to_string()); //push the keyword union
        head += 1;
        //push the struct id_name
        stream.push(lexeme[head].get_token_value()); //push the struct name
        let name = lexeme[head].get_token_value();
        stream.push("{".to_string());
        head += 2;
        let mut temp_lexeme: Vec<Token> = Vec::new();
        while lexeme[head].get_token_type() != RightCurlyBrace {
            while lexeme[head].get_token_type() != Semicolon {
                temp_lexeme.push(lexeme[head].clone());
                head += 1
            }
            temp_lexeme.push(lexeme[head].clone());
            head += 1;
            stream.append(&mut self.parse_union_inbody_decl(&temp_lexeme, &name));
            temp_lexeme.clear();
        }
        stream.push(lexeme[head].get_token_value() + "\n");

        stream
    }

    /* parse union type declarations
     * input : union tag_name var [;= ...]
     * output : let [mut] variant_name = Sometype_variant
     */
    fn parse_union_decl(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;

        stream.push(UNION_DECL.get_doc().to_string());

        //push the keyword let
        stream.push("let".to_string());
        if !self.strict {
            stream.push("mut".to_string());
        }

        stream.push(lexeme[head + 2].get_token_value());
        head += 3;
        while lexeme[head].get_token_type() != Semicolon {
            stream.push(lexeme[head].get_token_value());
            head += 1;
        }

        stream.push(";".to_string());

        stream
    }
    /* parse union body into Some type body
     * return rust stream
     */
    fn parse_union_inbody_decl(&mut self, lexeme: &Vec<Token>, name: &String) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head = 0;
        //push the identifier
        stream.push(lexeme[head + 1].get_token_value());
        stream.push("(".to_string());
        let mut struct_memt = CStructMem {
            identifier: "NONE".to_string(),
            name: name.clone(),
            member_type: TokenType::Others,
        };
        let mut rust_type = "RUST_TYPE".to_string();
        //push the type
        if let Some(rust_typ) = parse_type(lexeme[head].get_token_type(), Modifier::Default) {
            rust_type = rust_typ.clone();
            stream.push(rust_typ);
            struct_memt.member_type = lexeme[head].get_token_type();
            struct_memt.identifier = lexeme[head + 1].get_token_value();
        }
        head += 2;
        stream.push("),".to_string());
        //update struct member table (may require for analysis

        self.struct_mem.push(struct_memt.clone());
        while lexeme[head].get_token_type() != Semicolon {
            if lexeme[head].get_token_type() == Comma {
                head += 1;
            }
            struct_memt.identifier = lexeme[head].get_token_value();
            stream.push(lexeme[head].get_token_value());
            stream.push("(".to_string());
            stream.push(rust_type.clone());
            head += 1;
            stream.push("),".to_string());
            self.struct_mem.push(struct_memt.clone());
        }
        stream
    }

    // not tested
    fn parse_class(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head: usize = 0;
        let mut method_stream: Vec<String> = Vec::new();
        stream.push("struct".to_string()); //push the keyword struct
        head += 1;
        //push the struct id_name
        let class_name = lexeme[head].get_token_value();
        stream.push(class_name.clone()); //push the class name
        let name = lexeme[head].get_token_value();
        stream.push("{".to_string());
        head += 2;
        let mut modifier: String = " ".to_string();
        let mut temp_lexeme: Vec<Token> = Vec::new();
        let mut tstream: Vec<String> = Vec::new();

        while lexeme[head].get_token_type() != RightCurlyBrace
            && lexeme[head + 1].get_token_type() != Semicolon
        {
            match lexeme[head].get_type() {
                (TokenKind::Modifiers, _) => {
                    match lexeme[head].get_token_type() {
                        KeywordPublic => {
                            head += 2;
                            modifier = "pub".to_string();
                        }
                        KeywordProtected | keywordPrivate => {
                            head += 2;
                            modifier = "".to_string();
                        }
                        _ => {}
                    };
                }
                (_, Identifier) => {
                    if lexeme[head].get_token_value() == class_name {
                        tstream.push(CONSTRUCTOR.get_doc().to_string());
                        let mut lookahead = head;
                        while lexeme[lookahead].get_token_type() != LeftCurlyBrace {
                            lookahead += 1;
                        }
                        lookahead += 1;
                        lookahead = skip_block(lexeme, lookahead);
                        while head < lookahead {
                            tstream.push(lexeme[head].get_token_value());
                            head += 1;
                        }
                        tstream.push("\n **/\n".to_string());
                        continue;
                    }
                }

                _ => {}
            }

            if lexeme[head + 2].get_token_type() == LeftBracket {
                while lexeme[head].get_token_type() != RightCurlyBrace {
                    temp_lexeme.push(lexeme[head].clone());
                    head += 1;
                }
                temp_lexeme.push(lexeme[head].clone());
                head += 1;
                method_stream.append(&mut self.parse_method_decl(&temp_lexeme, &modifier));
                temp_lexeme.clear();
            } else {
                while lexeme[head].get_token_type() != RightCurlyBrace
                    && lexeme[head].get_token_kind() != TokenKind::Modifiers
                {
                    while lexeme[head].get_token_type() != Semicolon {
                        temp_lexeme.push(lexeme[head].clone());
                        head += 1
                    }
                    temp_lexeme.push(lexeme[head].clone());
                    head += 1;
                    stream.append(&mut self.parse_class_inbody_decl(
                        &temp_lexeme,
                        &name,
                        &modifier,
                    ));
                    temp_lexeme.clear();
                }
            }
        }
        stream.push(lexeme[head].get_token_value());
        stream.push(
            "\n\n/**Method declarations are wrapped inside the impl block \
    \n * Which implements the corresponding structure\
    \n **/\n"
                .to_string(),
        );
        stream.push("impl".to_string());
        stream.push(name.clone());
        stream.push("{\n".to_string());
        if tstream.len() > 0 {
            stream.append(&mut tstream);
        }
        stream.append(&mut method_stream);

        stream.push("}\n".to_string());
        stream
    }

    // not tested
    fn parse_method_decl(&mut self, lexeme: &Vec<Token>, modifier: &String) -> Vec<String> {
        let mut temp_lexeme: Vec<Token> = Vec::new();
        let mut head: usize = 3;
        let mut lookahead: usize = head;
        let mut stream: Vec<String> = Vec::new();
        if modifier.len() > 1 {
            stream.push(modifier.clone());
        }
        stream.push("fn".to_string());
        stream.push(lexeme[1].get_token_value());
        stream.push("(".to_string());
        stream.push("&self".to_string()); //first argument of method must be self, for sefety we consider reference/borrow
                                          // parse arguments differenly for functions that are not main
                                          // collect arguments
        while lexeme[lookahead].get_token_type() != RightBracket {
            lookahead += 1;
        }
        if head < lookahead {
            stream.push(",".to_string());
        }
        while head < lookahead {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }

        // parse arguments
        stream.append(&mut self.parse_arguments(&temp_lexeme));
        temp_lexeme.clear();

        stream.push(")".to_string());

        // parse return type
        if let Some(rust_type) = parse_type(lexeme[0].get_token_type(), Modifier::Default) {
            if rust_type != "void".to_string() {
                stream.push("->".to_string());
                stream.push(rust_type);
            }
        }

        stream.push("{".to_string());
        while lexeme[head].get_token_type() != LeftCurlyBrace {
            head += 1
        }
        head += 1;

        // collect function body
        // len - 1  so that '}' is excluded
        while head < lexeme.len() - 1 {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse function body
        stream.append(&mut self.parse_program(&temp_lexeme));
        stream.push("}".to_string());
        stream
    }

    // not tested
    fn parse_class_inbody_decl(
        &mut self,
        lexeme: &Vec<Token>,
        name: &String,
        modifier: &String,
    ) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();
        let mut head = 0;
        //push the identifier
        if modifier.len() > 1 {
            stream.push(modifier.clone());
        }
        stream.push(lexeme[head + 1].get_token_value());
        stream.push(":".to_string());
        let mut struct_memt = CStructMem {
            identifier: "NONE".to_string(),
            name: name.clone(),
            member_type: TokenType::Others,
        };

        let mut rust_type: String = " ".to_string();
        if let Some(rust_typ) = parse_type(lexeme[0].get_token_type(), Modifier::Default) {
            rust_type = rust_typ.clone();
            stream.push(rust_typ);
            struct_memt.member_type = lexeme[0].get_token_type();
            struct_memt.identifier = lexeme[1].get_token_value();
        }

        stream.push(",".to_string());
        self.struct_mem.push(struct_memt.clone());
        head += 2;
        while lexeme[head].get_token_type() != Semicolon {
            if lexeme[head].get_token_type() == Comma {
                head += 1;
            }
            stream.push(lexeme[head].get_token_value());
            stream.push(":".to_string());
            stream.push(rust_type.clone());

            struct_memt.identifier = lexeme[head].get_token_value();
            self.struct_mem.push(struct_memt.clone());
            head += 1;
        }
        stream
    }

    // not tested
    fn parse_class_decl(&mut self, lexeme: &Vec<Token>) -> Vec<String> {
        let mut stream: Vec<String> = Vec::new();

        stream.push(STRUCT_INIT.get_doc().to_string());
        stream.push("let".to_string());
        let mut head = 0;
        //struct FilePointer fp;
        let struct_name = lexeme[head].get_token_value();
        head += 1;
        stream.push(lexeme[head].get_token_value()); //push the identifer => let a
        stream.push("=".to_string());
        stream.push(struct_name.clone());
        stream.push("{".to_string());

        for row in &self.struct_mem {
            if row.name == struct_name {
                stream.push(row.identifier.clone());
                stream.push(":".to_string());
                stream.push(get_default_value_for(row.member_type));
                stream.push(",".to_string());
            }
        }
        stream.push("};".to_string());

        stream
    }
}
