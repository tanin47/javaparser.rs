use analyze::resolve::scope::Scope;
use parse::tree::Expr;
use semantics::Context;

pub mod field_access;
pub mod name;

pub fn apply<'def, 'def_ref, 'scope_ref>(
    expr: &'def_ref Expr<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    match expr {
        Expr::FieldAccess(f) => field_access::apply(f, context),
        Expr::ArrayAccess(_) => {}
        Expr::ArrayInitializer(_) => {}
        Expr::Assignment(_) => {}
        Expr::BinaryOperation(_) => {}
        Expr::Boolean(_) => {}
        Expr::Cast(_) => {}
        Expr::Char(_) => {}
        Expr::ConstructorReference(_) => {}
        Expr::Double(_) => {}
        Expr::Float(_) => {}
        Expr::Hex(_) => {}
        Expr::InstanceOf(_) => {}
        Expr::Int(_) => {}
        Expr::Lambda(_) => {}
        Expr::Long(_) => {}
        Expr::MethodCall(_) => {}
        Expr::MethodReference(_) => {}
        Expr::Name(n) => name::apply(n, context),
        Expr::NewArray(_) => {}
        Expr::NewObject(_) => {}
        Expr::Null(_) => {}
        Expr::Class(_) => {}
        Expr::String(_) => {}
        Expr::Super(_) => {}
        Expr::SuperConstructorCall(_) => {}
        Expr::This(_) => {}
        Expr::ThisConstructorCall(_) => {}
        Expr::Ternary(_) => {}
        Expr::UnaryOperation(_) => {}
        Expr::StaticClass(_) => {}
    };
}
