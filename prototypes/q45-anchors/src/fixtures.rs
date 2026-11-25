//! The fixture revision pairs. Each pair has one expected result for each old line: the new
//! line, or outdated (`-`). Line numbers are 1-based.

pub struct Fixture {
    pub name: &'static str,
    pub old_path: &'static str,
    pub old: &'static [u8],
    /// The new path and content, from the Git rename detection. None when the file is deleted.
    pub new: Option<(&'static str, &'static [u8])>,
    pub expected: &'static str,
}

pub const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "line-unchanged",
        old_path: "src/a.rs",
        old: b"fn a() {\n    x();\n}\n\nfn b() {\n    x();\n}\n",
        new: Some(("src/a.rs", b"fn a() {\n    x();\n}\n\nfn b() {\n    x();\n}\n")),
        expected: "1:1 2:2 3:3 4:4 5:5 6:6 7:7",
    },
    Fixture {
        name: "lines-added-above",
        old_path: "src/a.rs",
        old: b"use a;\n\nfn main() {\n    run();\n    stop();\n}\n",
        new: Some(("src/a.rs", b"// one\n// two\n// three\nuse a;\n\nfn main() {\n    run();\n    stop();\n}\n")),
        expected: "1:4 2:5 3:6 4:7 5:8 6:9",
    },
    Fixture {
        name: "line-added-directly-above",
        old_path: "src/a.rs",
        old: b"use a;\n\nfn main() {\n    run();\n    stop();\n}\n",
        new: Some(("src/a.rs", b"use a;\n\nfn main() {\n    setup();\n    run();\n    stop();\n}\n")),
        expected: "1:1 2:2 3:3 4:5 5:6 6:7",
    },
    Fixture {
        name: "line-edited",
        old_path: "src/a.rs",
        old: b"fn main() {\n    let a = 1;\n    let b = 2;\n    run(a, b);\n}\n",
        new: Some(("src/a.rs", b"fn main() {\n    let a = 1;\n    let b = 3;\n    run(a, b);\n}\n")),
        expected: "1:1 2:2 3:- 4:4 5:5",
    },
    Fixture {
        name: "line-deleted",
        old_path: "src/a.rs",
        old: b"fn main() {\n    let a = 1;\n    let b = 2;\n    run(a, b);\n}\n",
        new: Some(("src/a.rs", b"fn main() {\n    let a = 1;\n    run(a, b);\n}\n")),
        expected: "1:1 2:2 3:- 4:3 5:4",
    },
    Fixture {
        name: "duplicate-blocks-one-deleted",
        old_path: "src/a.rs",
        old: b"start();\n    x();\n    y();\n}\n    x();\n    y();\n}\nstop();\n",
        new: Some(("src/a.rs", b"start();\n    x();\n    y();\n}\nstop();\n")),
        expected: "1:1 2:- 3:- 4:- 5:- 6:- 7:- 8:5",
    },
    Fixture {
        name: "duplicate-blocks-copy-inserted",
        old_path: "src/a.rs",
        old: b"start();\n    x();\n    y();\n}\nstop();\n",
        new: Some(("src/a.rs", b"start();\n    x();\n    y();\n}\n    x();\n    y();\n}\nstop();\n")),
        expected: "1:1 2:- 3:- 4:- 5:8",
    },
    Fixture {
        name: "duplicate-blocks-one-edited",
        old_path: "src/a.rs",
        old: b"p();\n    a();\n    b();\n    c();\nq();\n    a();\n    b();\n    c();\nr();\n",
        new: Some(("src/a.rs", b"p();\n    a();\n    B();\n    c();\nq();\n    a();\n    b();\n    c();\nr();\n")),
        expected: "1:1 2:2 3:- 4:4 5:5 6:6 7:7 8:8 9:9",
    },
    Fixture {
        name: "duplicate-blocks-slide",
        old_path: "src/a.rs",
        old: b"x();\n    a();\n    b();\n    c();\n    d();\ny();\n    a();\n    b();\n    c();\n    d();\nz();\n",
        new: Some(("src/a.rs", b"x();\n    a();\n    b();\n    c();\n    d();\nz();\n")),
        expected: "1:1 2:- 3:- 4:- 5:- 6:- 7:- 8:- 9:- 10:- 11:6",
    },
    Fixture {
        name: "common-line-between-unique",
        old_path: "src/a.rs",
        old: b"fn a() {\n}\nfn b() {\n}\n",
        new: Some(("src/a.rs", b"fn a() {\n}\n// new\nfn b() {\n}\n")),
        expected: "1:1 2:2 3:4 4:5",
    },
    Fixture {
        name: "file-renamed",
        old_path: "src/old.rs",
        old: b"fn main() {\n    run();\n}\n",
        new: Some(("src/new.rs", b"// moved\nfn main() {\n    run();\n}\n")),
        expected: "1:2 2:3 3:4",
    },
    Fixture {
        name: "file-deleted",
        old_path: "src/a.rs",
        old: b"fn main() {\n    run();\n}\n",
        new: None,
        expected: "1:- 2:- 3:-",
    },
    Fixture {
        name: "whitespace-on-anchor-line",
        old_path: "src/a.rs",
        old: b"fn main() {\n    let a = 1;\n    run(a);\n}\n",
        new: Some(("src/a.rs", b"fn main() {\n    let a = 1;\n        run(a);\n}\n")),
        expected: "1:1 2:2 3:- 4:4",
    },
    Fixture {
        name: "whitespace-on-other-line",
        old_path: "src/a.rs",
        old: b"fn main() {\n    let a = 1;\n    run(a);\n}\n",
        new: Some(("src/a.rs", b"fn main() {\n    let a = 1; \n    run(a);\n}\n")),
        expected: "1:1 2:- 3:3 4:4",
    },
    Fixture {
        name: "line-endings-lf-to-crlf",
        old_path: "src/a.rs",
        old: b"fn main() {\n    run();\n}\n",
        new: Some(("src/a.rs", b"fn main() {\r\n    run();\r\n}\r\n")),
        expected: "1:- 2:- 3:-",
    },
    Fixture {
        name: "line-endings-one-line",
        old_path: "src/a.rs",
        old: b"fn main() {\n    run();\n}\n",
        new: Some(("src/a.rs", b"fn main() {\n    run();\r\n}\n")),
        expected: "1:1 2:- 3:3",
    },
    Fixture {
        name: "no-newline-at-end",
        old_path: "src/a.rs",
        old: b"fn main() {\n    run();\n}\n",
        new: Some(("src/a.rs", b"fn main() {\n    run();\n}")),
        expected: "1:1 2:2 3:-",
    },
];

/// Parses `expected` into one result for each old line, 0-based.
pub fn expected(f: &Fixture, old_lines: usize) -> Vec<Option<usize>> {
    let mut out = vec![None; old_lines];
    let mut seen = 0;
    for item in f.expected.split_whitespace() {
        let (o, n) = item.split_once(':').expect("item");
        let o: usize = o.parse().expect("old line");
        out[o - 1] = if n == "-" { None } else { Some(n.parse::<usize>().expect("new line") - 1) };
        seen += 1;
    }
    assert_eq!(seen, old_lines, "{}: one expected result for each old line", f.name);
    out
}
