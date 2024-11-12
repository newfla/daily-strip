fn main() {
    #[cfg(feature = "slint_frontend")]
    slint_build::compile("src/frontend/slint-ui/main.slint").expect("Slint build failed");
}
