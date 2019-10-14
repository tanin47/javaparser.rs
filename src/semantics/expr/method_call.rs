use analyze::definition::{Method, Param};
use analyze::resolve::scope::Scope;
use parse::tree::{
    ClassType, Expr, InvocationContext, MethodCall, ParameterizedType, Type, TypeArg,
    TypeParamExtend,
};
use semantics::{expr, Context};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

enum ParamScore {
    Matched,
    Inherited,
    Coerced,
    Ok,
    Unmatched,
}

struct MethodWithScore<'a> {
    method: Method<'a>,
    param_scores: Vec<ParamScore>,
    unaccounted_arg_count: i32,
}

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
    let mut scores = vec![];

    for method in methods {
        scores.push(compute(method, method_call));
    }

    //    scores.sort_by(|a, b| )
}

fn realize_type<'def>(tpe: &Type<'def>, map: &HashMap<&str, Type<'def>>) -> Type<'def> {}

fn compute<'def, 'def_ref>(
    method: Method<'def>,
    &call: &'def_ref MethodCall<'def>,
) -> MethodWithScore<'def> {
    // If type_arg is specified, then there's no inference.
    let method = if let Some(type_args) = call.type_args_opt {
        realize_method_with_type_args(method, type_args)
    } else {
        method
    };

    let mut param_scores = vec![];
    for (param, arg) in method.params.iter().zip(call.args.iter()) {
        param_scores.push(compute_param_score(param, arg));
    }

    // TODO: handle var args

    MethodWithScore {
        method,
        param_scores,
        unaccounted_arg_count: (call.args.len() as i32) - (method.params.len() as i32),
    }
}

fn realize_method_with_type_args<'def>(
    method: Method<'def>,
    type_args: Vec<TypeArg<'def>>,
) -> Method<'def> {
    let mut map: HashMap<&str, Type<'def>> = HashMap::new();
    for (type_param, type_arg) in method.type_params.iter().zip(type_args.iter()) {
        map.insert(&type_param.name, type_arg.to_type())
    }
    let mut params = vec![];
    for param in &method.params {
        params.push(Param {
            tpe: RefCell::new(realize_type(param.tpe.borrow().deref(), &map)),
            name: param.name,
            is_varargs: param.is_varargs,
        })
    }
    Method {
        type_params: method.type_params,
        params,
        return_type: realize_type(&method.return_type, &map),
        depth: method.depth,
        def: method.def,
    }
}

fn compute_param_score<'def>(param: &Param<'def>, arg: &Expr<'def>) -> ParamScore {
    let arg_tpe = if let Some(arg_tpe) = arg.tpe_opt() {
        arg_tpe
    } else {
        return ParamScore::Matched;
    };
    match param.tpe.borrow().deref() {
        Type::Class(c) => compute_class_param_score(c, &arg_tpe),
        Type::Primitive(_) => {}
        Type::Array(_) => {}
        Type::Parameterized(_) => {}
        Type::UnknownType => ParamScore::Ok,
        Type::Void(_) => panic!(),
        Type::Wildcard(_) => panic!(),
    }
}

fn compute_parameterized_param_score<'def>(
    parameterized: &ParameterizedType<'def>,
    arg: &Type<'def>,
) -> ParamScore {
}

fn compute_class_param_score<'def>(class: &ClassType<'def>, arg: &Type<'def>) -> ParamScore {
    match arg {
        Type::Class(c) => {
            if c.def_opt == class.def_opt && c.def_opt.is_some() {
                return ParamScore::Matched;
            } else if c.inherits(class) {
                return ParamScore::Inherited;
            }
        }
        Type::Parameterized(p) => {
            let type_param = unsafe { &*p.def };

            if type_param.extends.borrow().is_empty() {
                return ParamScore::Inherited;
            }

            for extend in type_param.extends.borrow().iter() {
                let result = match extend {
                    TypeParamExtend::Class(c) => compute_class_param_score(class, arg),
                    TypeParamExtend::Parameterized(p) => compute_parameterized_param_score(p, arg),
                };

                if result != ParamScore::Unmatched {
                    return ParamScore::Inherited;
                }
            }
        }
        Type::Primitive(p) => {
            if can_coerce(class, p) {
                return ParamScore::Coerced;
            }
        }
        Type::Array(a) => (),
        Type::UnknownType => return ParamScore::Ok,
        Type::Void(_) => panic!(),
        Type::Wildcard(_) => panic!(),
    };

    ParamScore::Unmatched
}
