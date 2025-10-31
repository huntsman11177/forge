use chrono::{DateTime, Utc};
use serde_json::{Number, Value};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Value),
    Var(Vec<String>),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Error)]
pub enum ExprError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("unexpected token '{0}'")]
    UnexpectedToken(String),
    #[error("invalid literal: {0}")]
    InvalidLiteral(String),
    #[error("evaluation error: {0}")]
    Evaluation(String),
}

pub type ExprResult<T> = Result<T, ExprError>;

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    lexeme: String,
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Identifier,
    Number(f64),
    String,
    True,
    False,
    Null,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    AndAnd,
    OrOr,
    EqEq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    LParen,
    RParen,
    Comma,
    Dot,
    EOF,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(src: &str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
        }
    }

    fn lex(&mut self) -> ExprResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let at_end = token.kind == TokenKind::EOF;
            tokens.push(token);
            if at_end {
                break;
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> ExprResult<Token> {
        self.skip_whitespace();
        if self.is_at_end() {
            return Ok(Token {
                kind: TokenKind::EOF,
                lexeme: String::new(),
            });
        }

        let ch = self.advance();
        match ch {
            '0'..='9' => self.lex_number(ch),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(ch),
            '"' => self.lex_string(),
            '+' => Ok(self.simple_token(TokenKind::Plus, "+")),
            '-' => Ok(self.simple_token(TokenKind::Minus, "-")),
            '*' => Ok(self.simple_token(TokenKind::Star, "*")),
            '/' => Ok(self.simple_token(TokenKind::Slash, "/")),
            '!' => {
                if self.match_char('=') {
                    Ok(self.simple_token(TokenKind::NotEq, "!="))
                } else {
                    Ok(self.simple_token(TokenKind::Bang, "!"))
                }
            }
            '&' => {
                if self.match_char('&') {
                    Ok(self.simple_token(TokenKind::AndAnd, "&&"))
                } else {
                    Err(ExprError::UnexpectedToken("&".into()))
                }
            }
            '|' => {
                if self.match_char('|') {
                    Ok(self.simple_token(TokenKind::OrOr, "||"))
                } else {
                    Err(ExprError::UnexpectedToken("|".into()))
                }
            }
            '=' => {
                if self.match_char('=') {
                    Ok(self.simple_token(TokenKind::EqEq, "=="))
                } else {
                    Err(ExprError::UnexpectedToken("=".into()))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(self.simple_token(TokenKind::Lte, "<="))
                } else {
                    Ok(self.simple_token(TokenKind::Lt, "<"))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(self.simple_token(TokenKind::Gte, ">="))
                } else {
                    Ok(self.simple_token(TokenKind::Gt, ">"))
                }
            }
            '(' => Ok(self.simple_token(TokenKind::LParen, "(")),
            ')' => Ok(self.simple_token(TokenKind::RParen, ")")),
            ',' => Ok(self.simple_token(TokenKind::Comma, ",")),
            '.' => Ok(self.simple_token(TokenKind::Dot, ".")),
            other => Err(ExprError::UnexpectedToken(other.to_string())),
        }
    }

    fn simple_token(&self, kind: TokenKind, lexeme: &str) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
        }
    }

    fn lex_number(&mut self, first: char) -> ExprResult<Token> {
        let mut literal = String::from(first);
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            literal.push(self.advance());
        }
        if self.peek() == Some('.') {
            literal.push(self.advance());
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                literal.push(self.advance());
            }
        }
        let value: f64 = literal
            .parse()
            .map_err(|_| ExprError::InvalidLiteral(literal.clone()))?;
        Ok(Token {
            kind: TokenKind::Number(value),
            lexeme: literal,
        })
    }

    fn lex_identifier(&mut self, first: char) -> ExprResult<Token> {
        let mut literal = String::from(first);
        while self
            .peek()
            .map_or(false, |c| c.is_ascii_alphanumeric() || c == '_')
        {
            literal.push(self.advance());
        }
        let kind = match literal.as_str() {
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            _ => TokenKind::Identifier,
        };
        Ok(Token {
            kind,
            lexeme: literal,
        })
    }

    fn lex_string(&mut self) -> ExprResult<Token> {
        let mut literal = String::new();
        while let Some(ch) = self.peek() {
            self.advance();
            match ch {
                '"' => {
                    return Ok(Token {
                        kind: TokenKind::String,
                        lexeme: literal,
                    })
                }
                '\\' => {
                    if let Some(escaped) = self.peek() {
                        self.advance();
                        let resolved = match escaped {
                            '"' => '"',
                            '\\' => '\\',
                            '/' => '/',
                            'b' => '\u{0008}',
                            'f' => '\u{000C}',
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            other => {
                                return Err(ExprError::InvalidLiteral(format!(
                                    "unsupported escape \\{other}"
                                )))
                            }
                        };
                        literal.push(resolved);
                    } else {
                        return Err(ExprError::UnexpectedEof);
                    }
                }
                other => literal.push(other),
            }
        }
        Err(ExprError::UnexpectedEof)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.pos];
        self.pos += 1;
        ch
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
}

pub fn parse_expression(src: &str) -> ExprResult<Expr> {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.lex()?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expression()?;
    parser.expect(TokenKind::EOF)?;
    Ok(expr)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_expression(&mut self) -> ExprResult<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> ExprResult<Expr> {
        let mut expr = self.parse_and()?;
        while self.match_kind(TokenKind::OrOr) {
            let rhs = self.parse_and()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::Or, Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> ExprResult<Expr> {
        let mut expr = self.parse_equality()?;
        while self.match_kind(TokenKind::AndAnd) {
            let rhs = self.parse_equality()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::And, Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> ExprResult<Expr> {
        let mut expr = self.parse_comparison()?;
        loop {
            if self.match_kind(TokenKind::EqEq) {
                let rhs = self.parse_comparison()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Eq, Box::new(rhs));
            } else if self.match_kind(TokenKind::NotEq) {
                let rhs = self.parse_comparison()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::NotEq, Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> ExprResult<Expr> {
        let mut expr = self.parse_term()?;
        loop {
            if self.match_kind(TokenKind::Lt) {
                let rhs = self.parse_term()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Lt, Box::new(rhs));
            } else if self.match_kind(TokenKind::Lte) {
                let rhs = self.parse_term()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Lte, Box::new(rhs));
            } else if self.match_kind(TokenKind::Gt) {
                let rhs = self.parse_term()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Gt, Box::new(rhs));
            } else if self.match_kind(TokenKind::Gte) {
                let rhs = self.parse_term()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Gte, Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> ExprResult<Expr> {
        let mut expr = self.parse_factor()?;
        loop {
            if self.match_kind(TokenKind::Plus) {
                let rhs = self.parse_factor()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Add, Box::new(rhs));
            } else if self.match_kind(TokenKind::Minus) {
                let rhs = self.parse_factor()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Sub, Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> ExprResult<Expr> {
        let mut expr = self.parse_unary()?;
        loop {
            if self.match_kind(TokenKind::Star) {
                let rhs = self.parse_unary()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Mul, Box::new(rhs));
            } else if self.match_kind(TokenKind::Slash) {
                let rhs = self.parse_unary()?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Div, Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> ExprResult<Expr> {
        if self.match_kind(TokenKind::Bang) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryOp::Not, Box::new(expr)));
        }
        if self.match_kind(TokenKind::Minus) {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryOp::Negate, Box::new(expr)));
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> ExprResult<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.match_kind(TokenKind::LParen) {
                let args = self.parse_arguments()?;
                let callee_name = match expr {
                    Expr::Var(ref path) if path.len() == 1 => path[0].clone(),
                    Expr::Var(_) => {
                        return Err(ExprError::UnexpectedToken(
                            "method-style calls are not supported".into(),
                        ))
                    }
                    _ => {
                        return Err(ExprError::UnexpectedToken(
                            "call expression must start with identifier".into(),
                        ))
                    }
                };
                expr = Expr::Call(callee_name, args);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> ExprResult<Expr> {
        if self.match_kind(TokenKind::LParen) {
            let expr = self.parse_expression()?;
            self.expect(TokenKind::RParen)?;
            return Ok(expr);
        }

        match self.peek_kind().clone() {
            TokenKind::Number(value) => {
                self.advance();
                let number = Number::from_f64(value)
                    .ok_or_else(|| ExprError::InvalidLiteral(value.to_string()))?;
                Ok(Expr::Literal(Value::Number(number)))
            }
            TokenKind::String => {
                let lexeme = self.advance().lexeme;
                Ok(Expr::Literal(Value::String(lexeme)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(Value::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(Value::Bool(false)))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expr::Literal(Value::Null))
            }
            TokenKind::Identifier => self.parse_variable(),
            TokenKind::EOF => Err(ExprError::UnexpectedEof),
            other => Err(ExprError::UnexpectedToken(format!("{other:?}"))),
        }
    }

    fn parse_variable(&mut self) -> ExprResult<Expr> {
        let mut path = Vec::new();
        let ident = self.advance();
        path.push(ident.lexeme);
        while self.match_kind(TokenKind::Dot) {
            let next = self.expect(TokenKind::Identifier)?;
            path.push(next.lexeme);
        }
        Ok(Expr::Var(path))
    }

    fn parse_arguments(&mut self) -> ExprResult<Vec<Expr>> {
        let mut args = Vec::new();
        if self.match_kind(TokenKind::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expression()?);
            if self.match_kind(TokenKind::Comma) {
                continue;
            }
            self.expect(TokenKind::RParen)?;
            break;
        }
        Ok(args)
    }

    fn expect(&mut self, expected: TokenKind) -> ExprResult<Token> {
        let token = self.advance();
        if token.kind.kind_eq(&expected) {
            Ok(token)
        } else {
            Err(ExprError::UnexpectedToken(format!(
                "expected {:?}, got {:?}",
                expected, token.kind
            )))
        }
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind().kind_eq(&kind) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }
}

impl TokenKind {
    fn kind_eq(&self, other: &TokenKind) -> bool {
        use TokenKind::*;
        match (self, other) {
            (Identifier, Identifier)
            | (String, String)
            | (True, True)
            | (False, False)
            | (Null, Null)
            | (Plus, Plus)
            | (Minus, Minus)
            | (Star, Star)
            | (Slash, Slash)
            | (Bang, Bang)
            | (AndAnd, AndAnd)
            | (OrOr, OrOr)
            | (EqEq, EqEq)
            | (NotEq, NotEq)
            | (Lt, Lt)
            | (Lte, Lte)
            | (Gt, Gt)
            | (Gte, Gte)
            | (LParen, LParen)
            | (RParen, RParen)
            | (Comma, Comma)
            | (Dot, Dot)
            | (EOF, EOF) => true,
            (Number(_), Number(_)) => true,
            _ => false,
        }
    }
}

pub struct EvalContext<'a> {
    variables: &'a Value,
    now: DateTime<Utc>,
}

impl<'a> EvalContext<'a> {
    pub fn new(variables: &'a Value, now: DateTime<Utc>) -> Self {
        Self { variables, now }
    }

    pub fn with_now(variables: &'a Value) -> Self {
        Self {
            variables,
            now: Utc::now(),
        }
    }
}

pub fn eval_expression(expr: &Expr, ctx: &EvalContext<'_>) -> ExprResult<Value> {
    match expr {
        Expr::Literal(v) => Ok(v.clone()),
        Expr::Var(path) => Ok(lookup_variable(ctx.variables, path)),
        Expr::Unary(op, inner) => {
            let value = eval_expression(inner, ctx)?;
            match op {
                UnaryOp::Negate => {
                    let number = to_number(&value)?;
                    Ok(Value::Number(
                        serde_json::Number::from_f64(-number).unwrap(),
                    ))
                }
                UnaryOp::Not => Ok(Value::Bool(!truthy(&value))),
            }
        }
        Expr::Binary(left, op, right) => eval_binary(left, *op, right, ctx),
        Expr::Call(name, args) => eval_call(name, args, ctx),
    }
}

fn eval_binary(
    left: &Expr,
    op: BinaryOp,
    right: &Expr,
    ctx: &EvalContext<'_>,
) -> ExprResult<Value> {
    match op {
        BinaryOp::And => {
            let lhs = eval_expression(left, ctx)?;
            if !truthy(&lhs) {
                return Ok(Value::Bool(false));
            }
            let rhs = eval_expression(right, ctx)?;
            Ok(Value::Bool(truthy(&rhs)))
        }
        BinaryOp::Or => {
            let lhs = eval_expression(left, ctx)?;
            if truthy(&lhs) {
                return Ok(Value::Bool(true));
            }
            let rhs = eval_expression(right, ctx)?;
            Ok(Value::Bool(truthy(&rhs)))
        }
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
            let lhs = eval_expression(left, ctx)?;
            let rhs = eval_expression(right, ctx)?;
            let lnum = to_number(&lhs)?;
            let rnum = to_number(&rhs)?;
            let result = match op {
                BinaryOp::Add => lnum + rnum,
                BinaryOp::Sub => lnum - rnum,
                BinaryOp::Mul => lnum * rnum,
                BinaryOp::Div => {
                    if rnum == 0.0 {
                        return Err(ExprError::Evaluation("division by zero".into()));
                    }
                    lnum / rnum
                }
                _ => unreachable!(),
            };
            to_json_number(result)
        }
        BinaryOp::Eq | BinaryOp::NotEq => {
            let lhs = eval_expression(left, ctx)?;
            let rhs = eval_expression(right, ctx)?;
            let eq = lhs == rhs;
            Ok(Value::Bool(if op == BinaryOp::Eq { eq } else { !eq }))
        }
        BinaryOp::Lt | BinaryOp::Lte | BinaryOp::Gt | BinaryOp::Gte => {
            let lhs = eval_expression(left, ctx)?;
            let rhs = eval_expression(right, ctx)?;
            compare_values(op, &lhs, &rhs)
        }
    }
}

fn compare_values(op: BinaryOp, lhs: &Value, rhs: &Value) -> ExprResult<Value> {
    match (lhs, rhs) {
        (Value::Number(ln), Value::Number(rn)) => {
            let l = ln.as_f64().unwrap();
            let r = rn.as_f64().unwrap();
            let result = match op {
                BinaryOp::Lt => l < r,
                BinaryOp::Lte => l <= r,
                BinaryOp::Gt => l > r,
                BinaryOp::Gte => l >= r,
                _ => unreachable!(),
            };
            Ok(Value::Bool(result))
        }
        (Value::String(ls), Value::String(rs)) => {
            let result = match op {
                BinaryOp::Lt => ls < rs,
                BinaryOp::Lte => ls <= rs,
                BinaryOp::Gt => ls > rs,
                BinaryOp::Gte => ls >= rs,
                _ => unreachable!(),
            };
            Ok(Value::Bool(result))
        }
        _ => Err(ExprError::Evaluation(
            "comparison requires two numbers or two strings".into(),
        )),
    }
}

fn eval_call(name: &str, args: &[Expr], ctx: &EvalContext<'_>) -> ExprResult<Value> {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "concat" => {
            let mut result = String::new();
            for arg in args {
                let value = eval_expression(arg, ctx)?;
                result.push_str(&value_to_string(&value));
            }
            Ok(Value::String(result))
        }
        "len" => {
            if args.len() != 1 {
                return Err(ExprError::Evaluation(
                    "len() expects exactly one argument".into(),
                ));
            }
            let value = eval_expression(&args[0], ctx)?;
            let length = match value {
                Value::String(s) => s.chars().count() as f64,
                Value::Array(arr) => arr.len() as f64,
                _ => {
                    return Err(ExprError::Evaluation(
                        "len() supports only string or array arguments".into(),
                    ))
                }
            };
            to_json_number(length)
        }
        "now" => {
            if !args.is_empty() {
                return Err(ExprError::Evaluation("now() takes no arguments".into()));
            }
            Ok(Value::String(ctx.now.to_rfc3339()))
        }
        _ => Err(ExprError::Evaluation(format!("unknown function {name}"))),
    }
}

fn lookup_variable(root: &Value, path: &[String]) -> Value {
    let mut current = root;
    for segment in path {
        match current {
            Value::Object(map) => match map.get(segment) {
                Some(value) => current = value,
                None => return Value::Null,
            },
            _ => return Value::Null,
        }
    }
    current.clone()
}

fn to_number(value: &Value) -> ExprResult<f64> {
    match value {
        Value::Number(num) => num
            .as_f64()
            .ok_or_else(|| ExprError::Evaluation("invalid number".into())),
        _ => Err(ExprError::Evaluation("expected number".into())),
    }
}

fn truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n) => n.as_f64().map_or(false, |v| v != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(arr) => !arr.is_empty(),
        Value::Object(obj) => !obj.is_empty(),
    }
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn eval(src: &str, vars: Value) -> Value {
        let expr = parse_expression(src).expect("parse");
        let ctx = EvalContext::with_now(&vars);
        eval_expression(&expr, &ctx).expect("eval")
    }

    #[test]
    fn arithmetic_precedence() {
        let result = eval("1 + 2 * 3", Value::Null);
        assert_eq!(result, json!(7.0));
    }

    #[test]
    fn boolean_logic() {
        let vars = json!({"user": {"active": true, "age": 21}});
        let expr = parse_expression("user.age >= 18 && user.active").unwrap();
        let ctx = EvalContext::with_now(&vars);
        let result = eval_expression(&expr, &ctx).unwrap();
        assert_eq!(result, json!(true));
    }

    #[test]
    fn concat_function() {
        let vars = json!({"user": {"name": "Ada"}});
        let result = eval("concat(user.name, \" \", \"Lovelace\")", vars);
        assert_eq!(result, json!("Ada Lovelace"));
    }

    #[test]
    fn len_function_on_array() {
        let vars = json!({"items": [1, 2, 3]});
        let result = eval("len(items)", vars);
        assert_eq!(result, json!(3.0));
    }

    #[test]
    fn now_function() {
        let vars = Value::Null;
        let expr = parse_expression("now()").unwrap();
        let custom_now = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let ctx = EvalContext::new(&vars, custom_now);
        let result = eval_expression(&expr, &ctx).unwrap();
        assert_eq!(result, json!("2024-01-01T00:00:00+00:00"));
    }

    #[test]
    fn variable_missing_returns_null() {
        let vars = Value::Null;
        let result = eval("user.name", vars);
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn division_by_zero_errors() {
        let expr = parse_expression("1 / 0").unwrap();
        let ctx = EvalContext::with_now(&Value::Null);
        let err = eval_expression(&expr, &ctx).unwrap_err();
        assert!(matches!(err, ExprError::Evaluation(_)));
    }
}

fn to_json_number(value: f64) -> ExprResult<Value> {
    Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| ExprError::Evaluation("invalid number value".into()))
}
