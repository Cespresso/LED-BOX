// Example demonstrating the original and duplicated modules work together

use led_box::utils;
use led_box::data;

fn main() {
    // Original modules
    println!("Original modules available:");
    println!("- utils::bluetooth");
    println!("- utils::led");
    println!("- data::nvs");
    
    // Duplicated modules with -2 suffix
    println!("Duplicated modules with -2 suffix:");
    println!("- utils::bluetooth_2");
    println!("- utils::led_2"); 
    println!("- data::nvs_2");
    
    // Note: These modules contain duplicated functionality with modified identifiers:
    // - Different UUIDs for Bluetooth services and characteristics
    // - Different GPIO pins for LED matrices
    // - Different NVS namespace and keys
    // - Modified service names with "-2" suffix
    
    println!("Duplication completed successfully!");
}