use crate::ast::*;
use crate::lexer::{Token, Lexer};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.current() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, self.current()))
        }
    }

    pub fn parse_file(&mut self) -> Result<FileContent, String> {
        // check if data.xe or function
        if let Token::Let = self.current() {
            // data definition
            let data = self.parse_data()?;
            Ok(FileContent::Data(data))
        } else if let Token::Fn = self.current() {
            // function definition
            let func = self.parse_function()?;
            Ok(FileContent::Function(func))
        } else {
            Err(format!("Expected fn or let, got {:?}", self.current()))
        }
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(Token::Fn)?;

        let name = match self.current() {
            Token::Ident(s) => {
                let n = s.clone();
                self.advance();
                n
            }
            _ => return Err("Expected function name".to_string()),
        };

        self.expect(Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(Token::RParen)?;

        self.expect(Token::LBrace)?;
        let (stmts, return_expr) = self.parse_block()?;
        self.expect(Token::RBrace)?;

        Ok(Function {
            name,
            params,
            body: stmts,
            return_expr,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();

        if let Token::RParen = self.current() {
            return Ok(params);
        }

        loop {
            let name = match self.current() {
                Token::Ident(s) => {
                    let n = s.clone();
                    self.advance();
                    n
                }
                _ => return Err("Expected parameter name".to_string()),
            };

            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            params.push(Param { name, ty });

            match self.current() {
                Token::Comma => {
                    self.advance();
                }
                _ => break,
            }
        }

        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.current() {
            Token::Int => {
                self.advance();
                if let Token::LBracket = self.current() {
                    self.advance();
                    self.expect(Token::RBracket)?;
                    Ok(Type::IntArray)
                } else {
                    Ok(Type::Int)
                }
            }
            _ => Err(format!("Expected type, got {:?}", self.current())),
        }
    }

    fn parse_block(&mut self) -> Result<(Vec<Stmt>, Option<Box<Expr>>), String> {
        let mut stmts = Vec::new();
        let mut return_expr = None;

        while !matches!(self.current(), Token::RBrace | Token::Eof) {
            match self.current() {
                Token::Let => {
                    stmts.push(self.parse_let()?);
                }
                Token::Return => {
                    self.advance();
                    let expr = self.parse_expr()?;
                    self.expect(Token::Semicolon)?;
                    stmts.push(Stmt::Return(expr));
                }
                _ => {
                    let expr = self.parse_expr()?;
                    if matches!(self.current(), Token::Semicolon) {
                        self.advance();
                        stmts.push(Stmt::ExprStmt(expr));
                    } else {
                        // Last expression is return value
                        return_expr = Some(Box::new(expr));
                        break;
                    }
                }
            }
        }

        Ok((stmts, return_expr))
    }

    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Let)?;

        let name = match self.current() {
            Token::Ident(s) => {
                let n = s.clone();
                self.advance();
                n
            }
            _ => return Err("Expected variable name".to_string()),
        };

        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Equals)?;
        let value = self.parse_expr()?;
        self.expect(Token::Semicolon)?;

        Ok(Stmt::Let { name, ty, value })
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let mut expr = match self.current() {
            Token::IntLit(n) => {
                let n = *n;
                self.advance();
                Expr::Int(n)
            }
            Token::Dot => {
                self.advance();
                let name = match self.current() {
                    Token::Ident(s) => {
                        let n = s.clone();
                        self.advance();
                        n
                    }
                    _ => return Err("Expected field name after .".to_string()),
                };
                Expr::DataRef(name)
            }
            Token::Ident(s) => {
                let name = s.clone();
                self.advance();
                Expr::Var(name)
            }
            Token::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !matches!(self.current(), Token::RBracket) {
                    loop {
                        elements.push(self.parse_expr()?);
                        if matches!(self.current(), Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(Token::RBracket)?;
                Expr::Array(elements)
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                expr
            }
            _ => return Err(format!("Unexpected token: {:?}", self.current())),
        };

        // parse postfix (call, index)
        loop {
            match self.current() {
                Token::LParen => {
                    // function call
                    if let Expr::Var(name) = expr {
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect(Token::RParen)?;
                        expr = Expr::Call { name, args };
                    } else {
                        return Err("Only identifiers can be called".to_string());
                    }
                }
                Token::LBracket => {
                    // array index
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(Token::RBracket)?;
                    expr = Expr::Index {
                        target: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();

        if matches!(self.current(), Token::RParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expr()?);
            if matches!(self.current(), Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(args)
    }

    fn parse_data(&mut self) -> Result<DataDef, String> {
        let mut entries = Vec::new();

        while !matches!(self.current(), Token::Eof | Token::RBrace) {
            if matches!(self.current(), Token::Let) {
                self.advance();
                let name = match self.current() {
                    Token::Ident(s) => {
                        let n = s.clone();
                        self.advance();
                        n
                    }
                    _ => return Err("Expected variable name".to_string()),
                };

                self.expect(Token::Colon)?;
                self.expect(Token::Int)?;
                self.expect(Token::Equals)?;

                let value = match self.current() {
                    Token::IntLit(n) => {
                        let v = *n;
                        self.advance();
                        v
                    }
                    _ => return Err("Expected integer value".to_string()),
                };

                self.expect(Token::Semicolon)?;
                entries.push((name, value));
            } else {
                break;
            }
        }

        Ok(DataDef { entries })
    }
}

pub fn parse(input: &str) -> Result<FileContent, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse_file()
}
