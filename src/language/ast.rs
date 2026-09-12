use std::fmt::Display;
use std::ops::Range;


pub type UntypedProgram = Program<()>;
pub type TypedProgram = Program<Type>;

pub struct Program<T> (pub Vec<Statement<T>>);

#[derive(PartialEq, Eq, Clone)]
pub enum Type {
    Unit,
    Integer,
    String,
    Page,
    PDF,
    Function(Vec<Type>, Box<Type>)
}

pub enum Statement<T> {
    Assignment(String, Expression<T>, Range<usize>),
    ExpressionStatement(Expression<T>, Range<usize>)
}

pub enum Expression<T> {
    BuiltIn(BuiltInExpression, Range<usize>),
    IntegerLiteral(i32, Range<usize>),
    StringLiteral(String, Range<usize>),
    Variable(String, Range<usize>, T),
    FunctionCall(Box<Expression<T>>, Vec<Expression<T>>, Range<usize>, T),
    Index(Box<Expression<T>>, Box<Expression<T>>, Range<usize>, T),
    PDFConstructor(Vec<Expression<T>>, Range<usize>, T)
}

#[derive(Clone, Copy)]
pub enum BuiltInExpression {
    Read,
    Write,
    Concatenate
}


impl Expression<Type> {
    pub fn t(&self) -> Type {
        match self {
            Expression::BuiltIn(BuiltInExpression::Read, _) => Type::Function(vec![Type::String], Box::new(Type::PDF)),
            Expression::BuiltIn(BuiltInExpression::Write, _) => Type::Function(vec![Type::String, Type::PDF], Box::new(Type::Unit)),
            Expression::BuiltIn(BuiltInExpression::Concatenate, _) => Type::Function(vec![Type::PDF, Type::PDF], Box::new(Type::PDF)),
            Expression::StringLiteral(_, _) => Type::String,
            Expression::IntegerLiteral(_, _) => Type::Integer,
            Expression::Variable(_, _, t) => t.clone(),
            Expression::FunctionCall(_, _, _, t) => t.clone(),
            Expression::Index(_, _, _, _) => Type::Page,
            Expression::PDFConstructor(_, _, _) => Type::PDF
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::Integer => write!(f, "Integer"),
            Type::String => write!(f, "String"),
            Type::Page => write!(f, "Page"),
            Type::PDF => write!(f, "PDF"),
            Type::Function(argument_types, return_type) => {
                let arguments = argument_types.iter().map(Type::to_string).collect::<Vec<_>>().join(", ");
                write!(f, "({}) -> {}", arguments, *return_type)
            }
        }
    }
}
