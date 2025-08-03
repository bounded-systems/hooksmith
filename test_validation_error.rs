fn main() {
    let unused_variable = 42; // This will trigger a Clippy warning
    println!("Hello, world!");
    println!("Testing file watch functionality!"); // New line to trigger file watching
} 
