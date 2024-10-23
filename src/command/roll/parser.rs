use eyre::{bail, Result};
use regex::Regex;

/******************
    LEXER
******************/

#[derive(Clone, Debug)]
enum Token {
    Const(String),
    Die(String),
    Term(String),
    Factor(String),
    Group(String),
}

impl std::fmt::Display for Token {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        use Token::*;
        match self {
            Const(s) => write!(f, "{}", s),
            Die(s) => write!(f, "{}", s),
            Term(s) => write!(f, "{}", s),
            Factor(s) => write!(f, "{}", s),
            Group(s) => write!(f, "{}", s),
        }
    }
}

impl Token {
    /// Used to check if two objects are of the same variant regardless
    /// of what they have as contents.
    pub fn id(&self) -> i64 {
        use Token::*;
        match self {
            Const(_) => 0,
            Die(_) => 1,
            Term(_) => 2,
            Factor(_) => 3,
            Group(_) => 4,
        }
    }

    pub fn value(&self) -> &String {
        use Token::*;
        match self {
            Const(s) => s,
            Die(s) => s,
            Term(s) => s,
            Factor(s) => s,
            Group(s) => s,
        }
    }
}

fn tokenize_expr(raw: &str) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut sb: String = String::new();

    for ch in raw.chars() {
        match ch {
            '\t' | '\n' | '\r' | ' ' | '\u{85}' | '\u{A0}' => {
                continue; // Ignore Whitespace
            }

            '+' | '-' | '*' | '/' | '(' | ')' => {
                // The previous token is over. Parse it before working on the next one.
                if sb.chars().count() != 0 {
                    match lex_token(&sb) {
                        None => bail!(
                            "{} was not recognized as a valid number or dice expression (Code: 1)",
                            sb
                        ),
                        Some(tok) => tokens.push(tok),
                    }
                }

                // Having processed the previous token, determine what kind of token
                // the delimiting character is.
                match ch {
                    '(' | ')' => tokens.push(Token::Group(ch.to_string())),
                    '*' | '/' => tokens.push(Token::Factor(ch.to_string())),
                    '+' | '-' => tokens.push(Token::Term(ch.to_string())),
                    _ => panic!("Unreachable!"),
                }

                sb.clear();
                continue;
            }

            // Non-transition characters are just added to the token currently being built.
            _ => sb.push(ch),
        }
    }

    // Parse any remaining characters in the buffer
    // that may not have been terminated by an operator.
    if sb.chars().count() != 0 {
        match lex_token(&sb) {
            None => bail!(
                "{} was not recognized as a valid number or dice expression (Code: 2)",
                sb
            ),
            Some(tok) => tokens.push(tok),
        }
    }

    Ok(tokens)
}

/// LexToken parses either a die or value expression from a string.
/// Returns None if the token is not valid.
fn lex_token(token: &str) -> Option<Token> {
    let mut t = token.to_owned();
    // Check for a const valur expr.
    if token.chars().all(|c| c.is_ascii_digit()) {
        return Some(Token::Const(t));
    }

    // Check for a Die Value Expr
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^\d*d\d+$"#).unwrap();
    }
    if RE.is_match(token) {
        // Run the roll handler.
        if token.starts_with('d') {
            // If the left hand expression is empty, that
            // means it's an implied leading 1.
            t = "1".to_owned();
            t.push_str(token);
        }
        return Some(Token::Die(t));
    }
    None
}

/******************
    PARSER & AST
******************/

// DiceParser converts a dice expression token stream to
// an AST and evaluates it according to the following grammar:
/*
    Expr	=> Term
    Term	=> Factor  ([ '+' | '-' ]) Factor)*
    Factor 	=> Primary ([ '*' | '/' ] Primary)*
    Primary => '(' Expr ')' | DIE | NUMBER
*/

pub struct DiceParser {
    tokens: Vec<Token>,
    current: u64,
    errors: Vec<String>,
}

impl DiceParser {
    pub fn new(raw: impl AsRef<str>) -> Result<DiceParser> {
        Ok(DiceParser::from_tokens(tokenize_expr(raw.as_ref())?))
    }

    fn from_tokens(tokens: Vec<Token>) -> DiceParser {
        DiceParser {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }

    // Expr satisfies the rule `Expr => Term`.
    pub fn expr(&mut self) -> Box<dyn AstExpr> {
        self.term()
    }

    // Term satisfies the rule for `Term	=> Factor  ([ '+' | '-' ]) Factor)*`
    pub fn term(&mut self) -> Box<dyn AstExpr> {
        let mut expr = self.factor(); // Left value of the term.

        while self.check(Token::Term(String::new())) {
            let op = self.consume();
            let right = self.factor();
            expr = Box::new(AstOp {
                left: expr,
                right,
                op,
            });
        }

        expr
    }

    // Factor satisfies the rule for `Factor 	=> Primary ([ '*' | '/' ] Primary)*`
    fn factor(&mut self) -> Box<dyn AstExpr> {
        let mut expr = self.primary();

        while self.check(Token::Factor(String::new())) {
            let op = self.consume(); // A token
            let right = self.primary(); // An AstExpr
            expr = Box::new(AstOp {
                left: expr,
                right,
                op,
            });
        }

        expr
    }

    // Primary satisfies the rule for `Primary => '(' Expr ')' | DIE | NUMBER`
    fn primary(&mut self) -> Box<dyn AstExpr> {
        // If the current token is a Constant...
        if self.check(Token::Const(String::new())) {
            let t = self.consume();

            // This should never fail because the tokenizer verifies that
            // this kind of token is purely numeric.
            return Box::new(AstConst(t.value().parse::<i64>().unwrap()));
        }

        if self.check(Token::Die(String::new())) {
            let t = self.consume();

            let mut split_die: Vec<&str> = t.value().split('d').collect();

            // A valid die expression is one with 2 parts, and the second part must be both present and numeric.
            if (split_die.len() != 2) || !split_die[1].chars().all(|c| c.is_ascii_digit()) {
                self.errors.push(format!(
                    "\"{}\" was not recognized as a valid number or dice expression (Code: 3)",
                    t.value()
                ));
                return Box::new(AstConst(0));
            }

            // An empty first string indicates that the die is of the dormat `dXX`
            // in which case there is an implied preceding 1.
            if split_die[0].is_empty() {
                split_die[0] = "1";
            }

            // This should never fail because the tokenizer verifies that
            // this kind of token is purely numeric.
            let left = match split_die[0].parse::<i64>() {
                Ok(num) => num,
                Err(_) => {
                    self.errors.push(format!(
                        "\"{}\" NUMBER in dice expression was not purely numeric",
                        t.value()
                    ));
                    0
                }
            };

            let right = match split_die[1].parse::<i64>() {
                Ok(num) => num,
                Err(_) => {
                    self.errors.push(format!(
                        "\"{}\" NUMBER in dice expression was not purely numeric",
                        t.value()
                    ));
                    0
                }
            };

            return Box::new(AstDie { left, right });
        }

        if self.check(Token::Group(String::new())) && self.peek().value() == "(" {
            let _ = self.consume();

            // In the case of a group, recurse back to the lowest priority and build a new subtree.
            let expr = self.expr();
            // Expect a closing paren.
            if self.check(Token::Group(String::new())) && self.peek().value() == ")" {
                let _ = self.consume();
                return expr;
            }
            // Error, unmatched Paren.
            self.errors.push("Unmatched parenthesis".to_owned());
            return Box::new(AstConst(0));
        }

        self.errors.push("Could not parse input".to_owned());
        Box::new(AstConst(0))
    }

    // Consumes the current token if it matches the given type,
    // advancing the cursor and returning it. Otherwise does nothing.
    fn consume(&mut self) -> Token {
        if !self.is_at_end() {
            // Advance the cursor and return whatever was before it.
            self.current += 1;
            return self.tokens[self.current as usize - 1].clone();
        }
        // If we are at the end, then there's only one token left to consume.
        self.tokens[self.current as usize].clone()
    }

    // Returns whether the token is of the given type. Does not consume.
    fn check(&self, typ: Token) -> bool {
        self.peek().id() == typ.id()
    }

    // Get the current token without advancing nor consuming it.
    fn peek(&self) -> Token {
        self.tokens[self.current as usize].clone()
    }

    // Returns whether the parser has any more tokens to consume,
    // that is, if "current" field == len(tokens) - 1
    fn is_at_end(&self) -> bool {
        self.current == (self.tokens.len().saturating_sub(1)) as u64
    }
}

/// AstExpr is any object which can resolve itself
/// to a final sum and a set of rolls (if any)
pub trait AstExpr: Send + Sync {
    /// Eval returns a result and a "steps string"
    fn eval(&self) -> (i64, String);
}

struct AstConst(i64);

/// An AstConst's value is simply itself.
impl AstExpr for AstConst {
    fn eval(&self) -> (i64, String) {
        (self.0, format!("{}", self.0))
    }
}

/// A random roll expression.
struct AstDie {
    left: i64,
    right: i64,
}

/// AstDie 's value is rolled, 1-[right] rolled [left] times, then summed.
impl AstExpr for AstDie {
    fn eval(&self) -> (i64, String) {
        let mut sb = String::new();
        sb.push('[');

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut rolls = Vec::with_capacity(self.left as usize);
        for i in 0..self.left {
            let roll = rng.gen_range(1..=self.right);

            sb.push_str(&format!("{}", roll));
            if i != self.left - 1 {
                sb.push_str(", ");
            }

            rolls.push(roll);
        }
        sb.push(']');

        return (rolls.iter().sum(), sb);
    }
}

/// A computable arithmetic expression.
struct AstOp {
    left: Box<dyn AstExpr>,
    right: Box<dyn AstExpr>,
    op: Token,
}

/// Do the math.
impl AstExpr for AstOp {
    fn eval(&self) -> (i64, String) {
        let left = self.left.eval();
        let right = self.right.eval();

        let steps = format!("{} {} {}", left.1, self.op, right.1);

        // IF the lexer did its job then these should be the only possible values.
        use Token::*;
        match &self.op {
            Term(s) => match s.as_str() {
                "+" => (left.0.saturating_add(right.0), steps),
                "-" => (left.0.saturating_sub(right.0), steps),
                _ => panic!("Unreachable! The Lexer produced a TERM with value {}", s),
            },
            Factor(s) => match s.as_str() {
                "*" => (left.0.saturating_mul(right.0), steps),
                "/" => {
                    if right.0 == 0 {
                        (0, "ERROR: DIVIDE BY ZERO".to_string())
                    } else {
                        (left.0.saturating_div(right.0), steps)
                    }
                }
                _ => panic!("Unreachable! The Lexer produced a FACTOR with value {}", s),
            },
            _ => panic!("Unreachable! The Lexer failed to validate an Op Token!"),
        }
    }
}
