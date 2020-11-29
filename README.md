# iter\_to\_array

⚠️ WARNING: This library contains `unsafe` code and has not been fully reviewed. ⚠️

Provides some traits for collecting iterators into arrays: `ToArray`, `ToArrayDefault` and `ToArrayPad`.

Also provides `Chunks` and `ChunksDefault` for iterating compile-time sized chunks.

This library uses unstable features (namely `const_generics`), so it cannot be used with stable Rust.

Usage example:

```rust
let iter = 0..5;

let arr: [5; i32] = iter.to_array().unwrap();
assert_eq!(arr, [0,1,2,3,4,5]);

let mut iter = 0..10;
let arr1: [5; i32] = iter.take_array().unwrap();  // only consumes as many elements as needed to fill the array
let arr2: [5; i32] = iter.take_array().unwrap();
assert_eq!(arr1, [0,1,2,3,4]);
assert_eq!(arr2, [5,6,7,8,9]);


// compile-time sized chunks iteration
for chunk in iter.chunks_default::<5>() {
    println!("chunk is {}", chunk);
}
```
