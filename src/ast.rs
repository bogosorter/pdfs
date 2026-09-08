use std::fmt::Display;

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
    Assignment(String, Expression<T>),
    ExpressionStatement(Expression<T>)
}

pub enum Expression<T> {
    StringLiteral(String),
    Variable(String, T),
    FunctionCall(Box<Expression<T>>, Vec<Expression<T>>, T)
}

impl Expression<Type> {
    pub fn t(&self) -> Type {
        match self {
            Expression::StringLiteral(_) => Type::String,
            Expression::Variable(_, t) => t.clone(),
            Expression::FunctionCall(_, _, t) => t.clone()
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
