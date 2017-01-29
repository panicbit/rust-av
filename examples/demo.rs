extern crate av;

fn main() {
    unsafe {
        av::encode_demo().unwrap();
    }
}
