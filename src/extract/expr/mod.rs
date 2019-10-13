use extract::Overlay;
use parse::tree::Expr;

pub mod field_access;
pub mod name;
pub mod static_class;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    expr: &'def_ref Expr<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    match expr {
        Expr::FieldAccess(f) => field_access::apply(f, overlay),
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
        Expr::Name(n) => name::apply(n, overlay),
        Expr::NewArray(_) => {}
        Expr::NewObject(_) => {}
        Expr::Null(_) => {}
        Expr::Class(_) => {}
        Expr::StaticClass(s) => static_class::apply(s, overlay),
        Expr::String(_) => {}
        Expr::Super(_) => {}
        Expr::SuperConstructorCall(_) => {}
        Expr::This(_) => {}
        Expr::ThisConstructorCall(_) => {}
        Expr::Ternary(_) => {}
        Expr::UnaryOperation(_) => {}
    }
}
