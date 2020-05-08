use roma::Router;

fn main() {

    let mut tree = Router::builder();

    let paths = vec![
        // "/rockets/{id}.{ext}",
        // "/foo/bar",
        // "{{WEIRD}}",
        "/app/installations",
        "/app/installations/{installation_id}",
        "/app/installations/{installation_id}/access_tokens",
        "/repos/{owner}/{repo}/check-runs",
        "/repos/{owner}/{repo}/check-runs/{check_run_id}",
        "/repos/{owner}/{repo}/check-runs/{check_run_id}/annotations",
        "/repos/{owner}/{repo}/check-suites",
        "/repos/{owner}/{repo}/check-suites/preferences",
        "/repos/{owner}/{repo}/check-suites/{check_suite_id}",
        "/repos/{owner}/{repo}/check-suites/{check_suite_id}/check-runs",
        "/repos/{owner}/{repo}/check-suites/{check_suite_id}/rerequest",
    ];

    for (i, r) in paths.iter().enumerate() {
        tree.insert(r, i);
    }

    let tree = tree.finish().unwrap();

    // let mut tree = Router::builder()
    //     .insert("", 0)
    //     .insert("/rockets/{id}.{ext}", 1)
    //     .insert("/GET/I/J", 3)
    //     .finish()
    //     .unwrap();

    println!("{:#?}", &tree);

    println!("{:#?}", &tree.find("/repos/rust-lang/rust/check-suites/610/check-runs".as_bytes()));

}