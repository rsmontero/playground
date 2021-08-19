mod messages;

fn main() {
    let _m = messages::TestMessage::parse(&String::from("Red 124 SUCCESS aGVsbG8gd29ybGQ=")).unwrap();
    println!("{:?}", _m);
}
