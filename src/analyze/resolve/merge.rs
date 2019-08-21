use analyze::definition::{Class, Interface, Package, Root};
use std::collections::HashMap;

pub fn apply(roots: Vec<Root>) -> Root {
    let mut subpackages = vec![];
    let mut units = vec![];

    for root in roots {
        for p in root.subpackages {
            subpackages.push(p);
        }

        for u in root.units {
            units.push(u);
        }
    }

    Root {
        subpackages: merge_packages(subpackages),
        units,
    }
}

fn merge_packages(packages: Vec<Package>) -> Vec<Package> {
    let mut by_name: HashMap<String, Vec<Package>> = HashMap::new();

    for p in packages {
        if !by_name.contains_key(p.name.as_str()) {
            by_name.insert(p.name.clone(), vec![]);
        }

        by_name.get_mut(&p.name).unwrap().push(p);
    }

    let mut result = vec![];

    for (_, ps) in by_name {
        let import_path = ps.get(0).unwrap().import_path.clone();
        let name = ps.get(0).unwrap().name.clone();

        let mut subpackages = vec![];
        let mut units = vec![];

        for p in ps {
            for s in p.subpackages {
                subpackages.push(s);
            }

            for u in p.units {
                units.push(u);
            }
        }

        result.push(Package {
            import_path,
            name,
            subpackages: merge_packages(subpackages),
            units,
        })
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

//#[cfg(test)]
//mod tests {
//    use super::apply;
//    use analyze::definition::{Class, Package, Root};
//    use analyze::test_common::mock_class;
//    use test_common::span;
//
//    #[test]
//    fn test_overall() {
//        assert_eq!(
//            apply(vec![Root {
//                subpackages: vec![
//                    Package {
//                        import_path: "dev".to_string(),
//                        name: "dev".to_string(),
//                        subpackages: vec![
//                            Package {
//                                import_path: "sub".to_string(),
//                                name: "sub".to_string(),
//                                subpackages: vec![],
//                                classes: vec![mock_class(&span(1, 1, "Test"))],
//                                interfaces: vec![]
//                            },
//                            Package {
//                                import_path: "sub2".to_string(),
//                                name: "sub2".to_string(),
//                                subpackages: vec![],
//                                classes: vec![],
//                                interfaces: vec![]
//                            }
//                        ],
//                        classes: vec![],
//                        interfaces: vec![]
//                    },
//                    Package {
//                        import_path: "dev".to_string(),
//                        name: "dev".to_string(),
//                        subpackages: vec![
//                            Package {
//                                import_path: "sub".to_string(),
//                                name: "sub".to_string(),
//                                subpackages: vec![],
//                                classes: vec![mock_class(&span(1, 1, "Test2"))],
//                                interfaces: vec![]
//                            },
//                            Package {
//                                import_path: "sub3".to_string(),
//                                name: "sub3".to_string(),
//                                subpackages: vec![],
//                                classes: vec![],
//                                interfaces: vec![]
//                            }
//                        ],
//                        classes: vec![],
//                        interfaces: vec![]
//                    },
//                    Package {
//                        import_path: "another".to_string(),
//                        name: "another".to_string(),
//                        subpackages: vec![],
//                        classes: vec![],
//                        interfaces: vec![]
//                    },
//                ],
//                classes: vec![],
//                interfaces: vec![]
//            }]),
//            Root {
//                subpackages: vec![
//                    Package {
//                        import_path: "another".to_string(),
//                        name: "another".to_string(),
//                        subpackages: vec![],
//                        classes: vec![],
//                        interfaces: vec![]
//                    },
//                    Package {
//                        import_path: "dev".to_string(),
//                        name: "dev".to_string(),
//                        subpackages: vec![
//                            Package {
//                                import_path: "sub".to_string(),
//                                name: "sub".to_string(),
//                                subpackages: vec![],
//                                classes: vec![
//                                    mock_class(&span(1, 1, "Test")),
//                                    mock_class(&span(1, 1, "Test2")),
//                                ],
//                                interfaces: vec![]
//                            },
//                            Package {
//                                import_path: "sub2".to_string(),
//                                name: "sub2".to_string(),
//                                subpackages: vec![],
//                                classes: vec![],
//                                interfaces: vec![]
//                            },
//                            Package {
//                                import_path: "sub3".to_string(),
//                                name: "sub3".to_string(),
//                                subpackages: vec![],
//                                classes: vec![],
//                                interfaces: vec![]
//                            }
//                        ],
//                        classes: vec![],
//                        interfaces: vec![]
//                    },
//                ],
//                classes: vec![],
//                interfaces: vec![]
//            }
//        )
//    }
//}
