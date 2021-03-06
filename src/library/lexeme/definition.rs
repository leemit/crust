pub const BLACK_HOLE: char = ' ';

//TokenKind kind of a token, grouped together based on usage and various characteristics of tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    DataTypes,
    BinaryOperators,
    UnaryOperators,
    AssignmentOperators,
    Comments,
    Values,
    Modifiers,
    Preprocessors,
    Identifiers,
    Typedef,
    Keyword,
    SpecialChars,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TokenType {
    //primitive types
    Integer,
    Short,
    Long,
    Float,
    Double,
    Character,
    Boolean,
    Void,
    Typedef,
    Unsigned,
    Signed,
    Auto,

    //operators token types
    Equal,
    NotEqual,
    Plus,
    Minus,
    Multiplication,
    Divide,
    Module,
    PlusEqual,
    MinusEqual,
    MultiplyEqual,
    DivideEqual,
    ModuleEqual,
    BitwiseAnd,
    BitwiseOr,
    BitwiseNegate,
    BitwiseLeftShift,
    BitwiseRightShift,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    Increment,
    Decrement,

    GreaterThan,
    LessThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    AddressOf,
    Assignment,
    //sizeof()
    SizeOf,
    TernaryOpetator,
    //---> Values type
    CharValue,
    StringValue,
    NumberInteger,
    NumberFloat,
    True,
    False,

    //special character token types
    LeftCurlyBrace,
    RightCurlyBrace,
    LeftBracket,
    RightBracket,
    LeftSquareBracket,
    RightSquareBracket,
    Colon,
    Semicolon,
    Comma,
    SingleLineComment,
    MultilineComment,
    Identifier,
    Arrow,
    ScopeResolution,

    //Identifiers an=d keywords
    KeywordClass,
    KeywordNew,
    KeywordFor,
    KeywordWhile,
    KeywordDo,
    KeywordBreak,
    KeywordContinue,
    KeywordSwitch,
    KeywordIf,
    KeywordElse,
    KeywordPublic,
    keywordPrivate,
    KeywordProtected,
    KeywordCase,
    KeywordStatic,
    KeywordConst,
    KeywordDefault,
    KeywordReturn,
    KeywordStruct,
    KeywordEnum,
    KeywordUnion,

    HeaderDefine,
    HeaderInclude,
    HeaderIfDefineStart,
    HeaderIfDefineEnd,
    Main,
    Null,
    Others,
}
