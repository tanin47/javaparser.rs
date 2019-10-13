use analyze::definition::Package;
use extract::{Definition, Overlay, Usage};
use parse::tree::PackagePrefix;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    package: &'def_ref PackagePrefix<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(prefix) = &package.prefix_opt {
        apply(prefix, overlay);
    }

    let def = unsafe { &*package.def };

    if let Some(span) = &package.span_opt {
        overlay.usages.push(Usage {
            span: span.clone(),
            def: Definition::Package(def),
        })
    }
}
