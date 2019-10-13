use analyze::resolve::scope::Scope;
use parse::tree::{InvocationContext, MethodCall};
use semantics::{expr, Context};

pub fn apply<'def, 'def_ref, 'scope_ref>(
    method_call: &'def_ref MethodCall<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    let invocation_context = InvocationContext { only_static: false };
    let methods = if let Some(prefix) = &method_call.prefix_opt {
        expr::apply(prefix, context);

        if let Some(tpe) = prefix.tpe_opt() {
            tpe.find_methods(method_call.name.fragment, &invocation_context)
        } else {
            vec![]
        }
    } else {
        context
            .scope
            .resolve_methods(method_call.name.fragment, &invocation_context);
    };

    // TODO: Score method based on args
}
