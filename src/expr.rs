//! Expression parsing and evaluation module for numeric expressions.
//!
//! This module provides a simple expression parser that supports:
//! - Basic arithmetic operators: +, -, *, /, %
//! - Parentheses for grouping
//! - Integer and floating-point numbers
//!
//! # Examples
//!
//! ```
//! use textcad::expr::Parser;
//!
//! let result = Parser::new("10 % 3").parse_and_eval().unwrap();
//! assert_eq!(result, 1.0);
//!
//! let result = Parser::new("(5 + 3) * 2").parse_and_eval().unwrap();
//! assert_eq!(result, 16.0);
//! ```

use crate::error::{Result, TextCadError};

/// Tokens produced by the lexer
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    LeftParen,
    RightParen,
    Eof,
}

/// Abstract Syntax Tree node for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal
    Number(f64),
    /// Binary operation: left op right
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    /// Unary operation: op expr
    UnaryOp { op: UnaryOperator, expr: Box<Expr> },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Plus,
}

/// Lexer for tokenizing expressions
struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<f64> {
        let start = self.position;
        let mut has_dot = false;

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        let num_str: String = self.input[start..self.position].iter().collect();
        num_str
            .parse::<f64>()
            .map_err(|_| TextCadError::SolverError(format!("Invalid number: {}", num_str)))
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        match self.current_char() {
            None => Ok(Token::Eof),
            Some('+') => {
                self.advance();
                Ok(Token::Plus)
            }
            Some('-') => {
                self.advance();
                Ok(Token::Minus)
            }
            Some('*') => {
                self.advance();
                Ok(Token::Multiply)
            }
            Some('/') => {
                self.advance();
                Ok(Token::Divide)
            }
            Some('%') => {
                self.advance();
                Ok(Token::Modulo)
            }
            Some('(') => {
                self.advance();
                Ok(Token::LeftParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RightParen)
            }
            Some(ch) if ch.is_ascii_digit() || ch == '.' => {
                let num = self.read_number()?;
                Ok(Token::Number(num))
            }
            Some(ch) => Err(TextCadError::SolverError(format!(
                "Unexpected character: {}",
                ch
            ))),
        }
    }
}

/// Parser for arithmetic expressions with operator precedence
pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    /// Create a new parser for the given input string
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token().unwrap_or(Token::Eof);
        Self {
            lexer,
            current_token,
        }
    }

    fn advance(&mut self) -> Result<()> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.current_token == expected {
            self.advance()?;
            Ok(())
        } else {
            Err(TextCadError::SolverError(format!(
                "Expected {:?}, found {:?}",
                expected, self.current_token
            )))
        }
    }

    /// Parse the expression and return the AST
    pub fn parse(&mut self) -> Result<Expr> {
        self.parse_expression()
    }

    /// Parse and evaluate the expression in one step
    pub fn parse_and_eval(mut self) -> Result<f64> {
        let expr = self.parse()?;
        self.expect(Token::Eof)?;
        expr.eval()
    }

    // Expression grammar with operator precedence:
    // expression  → term (('+' | '-') term)*
    // term        → factor (('*' | '/' | '%') factor)*
    // factor      → ('+' | '-')? primary
    // primary     → NUMBER | '(' expression ')'

    fn parse_expression(&mut self) -> Result<Expr> {
        let mut left = self.parse_term()?;

        while matches!(self.current_token, Token::Plus | Token::Minus) {
            let op = match self.current_token {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_term()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut left = self.parse_factor()?;

        while matches!(
            self.current_token,
            Token::Multiply | Token::Divide | Token::Modulo
        ) {
            let op = match self.current_token {
                Token::Multiply => BinaryOperator::Multiply,
                Token::Divide => BinaryOperator::Divide,
                Token::Modulo => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_factor()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        match self.current_token {
            Token::Plus => {
                self.advance()?;
                let expr = self.parse_primary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOperator::Plus,
                    expr: Box::new(expr),
                })
            }
            Token::Minus => {
                self.advance()?;
                let expr = self.parse_primary()?;
                Ok(Expr::UnaryOp {
                    op: UnaryOperator::Negate,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.current_token.clone() {
            Token::Number(n) => {
                self.advance()?;
                Ok(Expr::Number(n))
            }
            Token::LeftParen => {
                self.advance()?;
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(TextCadError::SolverError(format!(
                "Unexpected token: {:?}",
                self.current_token
            ))),
        }
    }
}

impl Expr {
    /// Evaluate the expression and return the result
    pub fn eval(&self) -> Result<f64> {
        match self {
            Expr::Number(n) => Ok(*n),
            Expr::BinaryOp { left, op, right } => {
                let left_val = left.eval()?;
                let right_val = right.eval()?;

                match op {
                    BinaryOperator::Add => Ok(left_val + right_val),
                    BinaryOperator::Subtract => Ok(left_val - right_val),
                    BinaryOperator::Multiply => Ok(left_val * right_val),
                    BinaryOperator::Divide => {
                        if right_val == 0.0 {
                            Err(TextCadError::SolverError("Division by zero".to_string()))
                        } else {
                            Ok(left_val / right_val)
                        }
                    }
                    BinaryOperator::Modulo => {
                        if right_val == 0.0 {
                            Err(TextCadError::SolverError("Modulo by zero".to_string()))
                        } else {
                            Ok(left_val % right_val)
                        }
                    }
                }
            }
            Expr::UnaryOp { op, expr } => {
                let val = expr.eval()?;
                match op {
                    UnaryOperator::Negate => Ok(-val),
                    UnaryOperator::Plus => Ok(val),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_number() {
        let result = Parser::new("42").parse_and_eval().unwrap();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_addition() {
        let result = Parser::new("2 + 3").parse_and_eval().unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_subtraction() {
        let result = Parser::new("10 - 3").parse_and_eval().unwrap();
        assert_eq!(result, 7.0);
    }

    #[test]
    fn test_multiplication() {
        let result = Parser::new("4 * 5").parse_and_eval().unwrap();
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_division() {
        let result = Parser::new("20 / 4").parse_and_eval().unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_modulo() {
        let result = Parser::new("10 % 3").parse_and_eval().unwrap();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_modulo_float() {
        let result = Parser::new("10.5 % 3.0").parse_and_eval().unwrap();
        assert_eq!(result, 1.5);
    }

    #[test]
    fn test_operator_precedence() {
        let result = Parser::new("2 + 3 * 4").parse_and_eval().unwrap();
        assert_eq!(result, 14.0); // 2 + (3 * 4) = 14
    }

    #[test]
    fn test_modulo_precedence() {
        let result = Parser::new("10 + 5 % 3").parse_and_eval().unwrap();
        assert_eq!(result, 12.0); // 10 + (5 % 3) = 10 + 2 = 12
    }

    #[test]
    fn test_parentheses() {
        let result = Parser::new("(2 + 3) * 4").parse_and_eval().unwrap();
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_nested_parentheses() {
        let result = Parser::new("((2 + 3) * 4) / 2").parse_and_eval().unwrap();
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_unary_minus() {
        let result = Parser::new("-5").parse_and_eval().unwrap();
        assert_eq!(result, -5.0);
    }

    #[test]
    fn test_unary_plus() {
        let result = Parser::new("+5").parse_and_eval().unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_complex_expression() {
        let result = Parser::new("2 + 3 * 4 - 10 / 2").parse_and_eval().unwrap();
        assert_eq!(result, 9.0); // 2 + 12 - 5 = 9
    }

    #[test]
    fn test_complex_with_modulo() {
        let result = Parser::new("15 % 4 * 2 + 1").parse_and_eval().unwrap();
        assert_eq!(result, 7.0); // (15 % 4) * 2 + 1 = 3 * 2 + 1 = 7
    }

    #[test]
    fn test_floating_point() {
        let result = Parser::new("3.5 + 2.5").parse_and_eval().unwrap();
        assert_eq!(result, 6.0);
    }

    #[test]
    fn test_whitespace() {
        let result = Parser::new("  2   +   3  ").parse_and_eval().unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_division_by_zero() {
        let result = Parser::new("10 / 0").parse_and_eval();
        assert!(result.is_err());
    }

    #[test]
    fn test_modulo_by_zero() {
        let result = Parser::new("10 % 0").parse_and_eval();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_syntax() {
        // "2 + * 3" is invalid - can't have multiply after plus without a primary expression
        let result = Parser::new("2 + * 3").parse_and_eval();
        assert!(result.is_err());
    }

    #[test]
    fn test_unclosed_parenthesis() {
        let result = Parser::new("(2 + 3").parse_and_eval();
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_modulo() {
        let result = Parser::new("100 % 30 % 7").parse_and_eval().unwrap();
        assert_eq!(result, 3.0); // (100 % 30) % 7 = 10 % 7 = 3
    }
}
