use std::fmt::Display;
use std::ops::Range;


pub type UntypedProgram = Program<()>;
pub type TypedProgram = Program<Type>;

pub struct Program<T> (pub Vec<Statement<T>>);

#[derive(PartialEq, Eq, Clone)]
pub enum Type {
    Unit,
    String,
    PDF,
    Function(Vec<Type>, Box<Type>)
}

pub enum Statement<T> {
    Assignment(String, Expression<T>, Range<usize>),
    ExpressionStatement(Expression<T>, Range<usize>)
}

pub enum Expression<T> {
    BuiltIn(BuiltInExpression, Range<usize>),
    StringLiteral(String, Range<usize>),
    Variable(String, Range<usize>, T),
    FunctionCall(Box<Expression<T>>, Vec<Expression<T>>, Range<usize>, T)
}

#[derive(Clone, Copy)]
pub enum BuiltInExpression {
    Read,
    Write
}


impl Expression<Type> {
    pub fn t(&self) -> Type {
        match self {
            Expression::BuiltIn(BuiltInExpression::Read, _) => Type::Function(vec![Type::String], Box::new(Type::PDF)),
            Expression::BuiltIn(BuiltInExpression::Write, _) => Type::Function(vec![Type::String, Type::PDF], Box::new(Type::Unit)),
            Expression::StringLiteral(_, _) => Type::String,
            Expression::Variable(_, _, t) => t.clone(),
            Expression::FunctionCall(_, _, _, t) => t.clone()
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::String => write!(f, "String"),
            Type::PDF => write!(f, "PDF"),
            Type::Function(argument_types, return_type) => {
                let arguments = argument_types.iter().map(Type::to_string).collect::<Vec<_>>().join(", ");
                write!(f, "({}) -> {}", arguments, *return_type)
            }
        }
    }
}
