pub enum Literal {
    Numeric(String),
}

pub enum Operator {
    Plus,
    Minus,
    Times,
    Divide,
    LogicalOr,
    LogicalAnd,
    LogicalNot,
    BinaryNot,
    BinaryOr,
    BinaryAnd,
    ShiftLeft,
    ShiftRight,
    ShiftRightRight,
    Inc,
    Dec,
    Mod,
    Pow,
}

pub enum AssignmentOperator {
    Equals,
    Plus,
    Minus,
    Times,
    Divide,
    Mod,
    Pow,
    ShiftLeft,
    ShiftRight,
    ShiftRightRight,
    BinaryAnd,
    BinaryOr,
    BinaryNot,
}

pub enum Punctuator {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Arrow,
    Question,
    Colon,
    Tilde,
    Dot,
    Ellipsis,
    Semicolon,
    Comma,
}

pub enum Comparator {
    LT,
    LTE,
    GT,
    GTE,
    EQ,
    NEQ,
    EEQ,
    NEEQ,
}

pub enum Node {
    Comment(bool, String),
    Whitespace(String),
    Operator(Operator),
    AssignmentOperator(AssignmentOperator),
    Punctuator(Punctuator),
    Identifier(String),
    Comparator(Comparator),
    Literal(Literal),
}
