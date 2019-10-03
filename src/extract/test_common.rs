use extract;
use extract::{Definition, Usage};
use parse::JavaFile;
use std::any::Any;
use std::collections::HashMap;
use std::iter::Zip;
use std::ops::Deref;

pub fn assert_extract(sources: Vec<&str>, expecteds: Vec<&str>) {
    let (files, _) = semantics_files!(vec sources);

    let mut usagess = vec![];
    let mut defs = vec![];
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

        for def in overlay.defs {
            if let Definition::Package(_) = def {
                // skip package
            } else {
                defs.push(def);
            }
        }
    }

    let mut file_rank: HashMap<*const JavaFile, usize> = HashMap::new();
    for (index, file) in files.iter().enumerate() {
        file_rank.insert(&**file, index);
    }

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

    let mut def_ids: HashMap<usize, usize> = HashMap::new();
    for (id, def) in defs.iter().enumerate() {
        def_ids.insert(def.ptr(), id);
    }

    let mut outputs = vec![];
    for (file, usages) in files.iter().zip(usagess.iter()) {
        let mut s = String::new();

        let usages: &[Usage] = usages;
        let mut usage_index = 0;

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

            s.push(c);

            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }

            if usage_index < usages.len() {
                let current_usage = &usages[usage_index];
                if current_usage.span.line == line
                    && (current_usage.span.col + current_usage.span.fragment.len()) == col
                {
                    s.push(']');
                    usage_index += 1;
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
