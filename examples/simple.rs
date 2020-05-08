use roma::Router;

fn main() {
    let mut tree = Router::builder()
        .insert("", 0)
        .insert("/rockets/{id}.{ext}", 1)
        .insert("/GET/I/J", 3)
        .finish()
        .unwrap();

    println!("{:#?}", &tree);

    println!("{:#?}", &tree.find("/rockets/😁.json".as_bytes()));

}