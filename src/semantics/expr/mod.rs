use analyze::resolve::scope::Scope;
use parse::tree::{Expr, Type};
use semantics::Context;

pub mod field_access;
pub mod lambda;
pub mod method_call;
pub mod name;

pub fn apply<'def>(
    expr: &mut Expr<'def>,
    target_type: &Type<'def>,
    context: &mut Context<'def, '_, '_>,
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
        Expr::Lambda(l) => lambda::apply(l, target_type, context),
        Expr::Long(_) => {}
        Expr::MethodCall(m) => method_call::apply(m, context),
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
