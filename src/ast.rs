pub type UntypedProgram = Program<()>;
pub type TypedProgram = Program<Type>;

pub struct Program<T> (pub Vec<Statement<T>>);

pub enum Type {
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

pub fn get_type(expression: Expression<Type>) -> Type {
    match expression {
        Expression::StringLiteral(_) => Type::String,
        Expression::Variable(_, t) => t,
        Expression::FunctionCall(_, _, t) => t
    }
}
