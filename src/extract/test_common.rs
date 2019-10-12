use analyze;
use analyze::test_common::find_class;
use extract::{Definition, Usage};
use std::any::Any;
use std::collections::HashMap;
use std::iter::Zip;
use std::ops::Deref;
use {extract, JavaFile};

pub fn assert_extract(sources: Vec<&str>, expecteds: Vec<&str>) {
    let (files, root) = apply_semantics!(vec sources);

    let mut usagess = vec![];
    let mut defss = vec![];
    for file in &files {
        let mut overlay = extract::apply(&file.unit);
        overlay.usages.sort_by(|a, b| {
            if a.span.line == b.span.line {
                a.span.col.cmp(&b.span.col)
            } else {
                a.span.line.cmp(&b.span.line)
            }
        });
        usagess.push(overlay.usages);

        let mut defs = vec![];
        for def in &overlay.defs {
            if let Definition::Package(_) = def {
                // skip package
            } else {
                defs.push(*def);
            }
        }
        defss.push(defs);
    }

    let mut file_rank: HashMap<*const JavaFile, usize> = HashMap::new();
    for (index, file) in files.iter().enumerate() {
        file_rank.insert(&**file, index);
    }

    let mut id = 0;
    let mut def_ids: HashMap<usize, usize> = HashMap::new();
    for defs in &mut defss {
        defs.sort_by(|a, b| {
            let a = a.span().unwrap();
            let b = b.span().unwrap();

            if a.file == b.file {
                if a.line == b.line {
                    a.col.cmp(&b.col)
                } else {
                    a.line.cmp(&b.line)
                }
            } else {
                (*file_rank.get(&a.file).unwrap()).cmp(file_rank.get(&b.file).unwrap())
            }
        });

        for def in defs {
            println!("{:#?}", def);
            if let Definition::Class(c) = &def {
                println!("HELLO");
                let c = unsafe { &**c };
                println!("{:#?}", c);
            }
            println!("{:#?}", def.span());
            def_ids.insert(def.ptr(), id);
            id += 1;
        }
    }

    let mut outputs = vec![];
    for ((file, usages), defs) in files.iter().zip(usagess.iter()).zip(defss.iter()) {
        let mut s = String::new();

        let usages: &[Usage] = usages;
        let mut usage_index = 0;

        let defs: &[Definition] = defs;
        let mut def_index = 0;

        let mut line = 1;
        let mut col = 1;

        for c in file.content.chars() {
            if usage_index < usages.len() {
                let current_usage = &usages[usage_index];
                if current_usage.span.line == line && current_usage.span.col == col {
                    s.push('[');
                    match def_ids.get(&current_usage.def.ptr()) {
                        Some(id) => {
                            s.push_str(&format!("{}", id.deref()));
                            s.push(':');
                        }
                        None => (),
                    };
                }
            }
            if def_index < defs.len() {
                let current = &defs[def_index];
                if current.span().unwrap().line == line && current.span().unwrap().col == col {
                    s.push('*');
                    match def_ids.get(&current.ptr()) {
                        Some(id) => {
                            s.push_str(&format!("{}", id.deref()));
                            s.push(':');
                        }
                        None => (),
                    };
                }
            }

            s.push(c);

            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }

            if usage_index < usages.len() {
                let current = &usages[usage_index];
                if current.span.line == line
                    && (current.span.col + current.span.fragment.len()) == col
                {
                    s.push(']');
                    usage_index += 1;
                }
            }

            if def_index < defs.len() {
                let current = &defs[def_index];
                if current.span().unwrap().line == line
                    && (current.span().unwrap().col + current.span().unwrap().fragment.len()) == col
                {
                    s.push('*');
                    def_index += 1;
                }
            }
        }

        outputs.push(s);
    }

    let mut sanitized_expecteds = vec![];
    for expected in expecteds {
        sanitized_expecteds.push(expected.trim());
    }

    assert_eq!(
        sanitized_expecteds,
        outputs,
        r#"
        
=======================
|      Expected       |
=======================
{}
=======================
|       Output        |
=======================
{}
"#,
        {
            let mut s = String::new();
            for expected in &sanitized_expecteds {
                s.push('\n');
                s.push_str(*expected);
                s.push('\n');
                s.push('\n');
                s.push_str("---------------------------------");
                s.push('\n');
            }
            s
        },
        {
            let mut s = String::new();
            for output in &outputs {
                s.push('\n');
                s.push_str(output);
                s.push('\n');
                s.push('\n');
                s.push_str("---------------------------------");
                s.push('\n');
            }
            s
        }
    );
}
