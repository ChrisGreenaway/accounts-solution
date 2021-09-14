use std::borrow::Cow;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::process::Output;

#[test]
fn no_args() {
    let output = run_args(vec![]);

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("Please provide the CSV input filename"));
}

#[test]
fn file_not_exists() {
    let output = run("missing");

    assert!(!output.status.success());
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("NotFound"));
}

#[test]
fn empty() {
    let output = run("empty.csv");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
}

#[test]
fn nonsense() {
    let output = run("nonsense.csv");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
}

#[test]
fn just_headers() {
    let output = run("just_headers.csv");

    assert!(output.status.success());
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
}

#[test]
fn simple() {
    let output = run("simple.csv");

    assert!(output.status.success());
    assert_eq!(stdout(&output), expect(vec!["1,1,0,1,false"]));
    assert_eq!(stderr(&output), "");
}

#[test]
fn extra_args() {
    let output = run_args(vec!["tests/simple.csv", "extra"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), expect(vec!["1,1,0,1,false"]));
    assert_eq!(stderr(&output), "");
}

#[test]
fn spaces() {
    let output = run("spaces.csv");

    assert!(output.status.success());
    assert_eq!(stdout(&output), expect(vec!["1,1,0,1,false"]));
    assert_eq!(stderr(&output), "");
}

#[test]
fn chargeback() {
    let output = run("chargeback.csv");

    assert!(output.status.success());
    assert_eq!(stdout(&output), expect(vec!["1,0,0,0,true"]));
    assert_eq!(stderr(&output), "");
}

#[test]
fn decimal() {
    let output = run("decimal.csv");

    assert!(output.status.success());
    assert_eq!(stdout(&output), expect(vec!["1,1.1577,0,1.1577,false"]));
    assert_eq!(stderr(&output), "");
}

#[test]
fn complex() {
    let output = run("complex.csv");

    assert!(output.status.success());
    compare(
        &stdout(&output),
        vec![
            "1,20.1,0,20.1,false",
            "2,222.22,200.22,422.44,false",
            "3,6333.666,0.000,6333.666,false",
            "4,44444.4444,0.0000,44444.4444,true",
        ],
    );
    assert_eq!(stderr(&output), "");
}

fn run(filename: &str) -> Output {
    let path = format!("tests/{}", filename);
    run_args(vec![&path])
}

fn run_args(args: Vec<&str>) -> Output {
    test_bin::get_test_bin("accounts-solution")
        .args(args)
        .output()
        .expect("Failed to find the binary")
}

fn stdout(output: &Output) -> Cow<str> {
    String::from_utf8_lossy(&output.stdout)
}

fn stderr(output: &Output) -> Cow<str> {
    String::from_utf8_lossy(&output.stderr)
}

fn expect(lines: Vec<&str>) -> String {
    let mut expect = String::from("client,available,held,total,locked\n");

    for line in lines {
        expect.push_str(line);
        expect.push('\n');
    }

    expect
}

fn compare(actual: &str, expected: Vec<&str>) {
    let mut actual = actual.lines().collect::<VecDeque<_>>();
    assert_eq!(
        actual.pop_front().unwrap(),
        "client,available,held,total,locked"
    );
    let actual: HashSet<&str> = HashSet::from_iter(actual.into_iter());
    let expected: HashSet<&str> = HashSet::from_iter(expected.into_iter());
    assert_eq!(actual, expected);
}
