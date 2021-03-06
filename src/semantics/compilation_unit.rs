use analyze::resolve::scope::{EnclosingTypeDef, Scope};
use parse::tree::CompilationUnitItem;
use semantics::def::class;
use semantics::{import, Context};
use {analyze, parse};

pub fn apply<'def>(
    unit: &mut parse::tree::CompilationUnit<'def>,
    context: &mut Context<'def, '_, '_>,
) {
    if let Some(package) = &unit.package_opt {
        enter_package(package, context);
    } else {
        context.scope.enter();
    }

    for im in &mut unit.imports {
        context.scope.add_import(im);
        import::apply(im, context);
    }

    for item in &mut unit.items {
        apply_item(item, context);
    }

    if let Some(package) = &unit.package_opt {
        leave_package(package, context);
    } else {
        context.scope.leave();
    }
}

fn apply_item<'def>(item: &mut CompilationUnitItem<'def>, context: &mut Context<'def, '_, '_>) {
    match item {
        CompilationUnitItem::Class(c) => class::apply(c, context),
        CompilationUnitItem::Interface(_) => panic!(),
        CompilationUnitItem::Annotation(_) => panic!(),
        CompilationUnitItem::Enum(_) => panic!(),
    };
}

fn enter_package<'def>(package: &parse::tree::Package<'def>, context: &mut Context<'def, '_, '_>) {
    if let Some(prefix) = &package.prefix_opt {
        enter_package(prefix, context);
        match context
            .scope
            .levels
            .last()
            .unwrap()
            .enclosing_opt
            .as_ref()
            .unwrap()
        {
            &EnclosingTypeDef::Package(p) => {
                context
                    .scope
                    .enter_package(unsafe { &*p }.find_package(package.name.fragment).unwrap());
            }
            _ => panic!(),
        }
    } else {
        context.scope.enter_package(unsafe {
            &*context
                .scope
                .resolve_package(package.name.fragment)
                .unwrap()
        });
    }
}

fn leave_package<'def>(package: &parse::tree::Package<'def>, context: &mut Context<'def, '_, '_>) {
    if let Some(prefix) = &package.prefix_opt {
        leave_package(prefix, context);
    }

    context.scope.leave();
}
