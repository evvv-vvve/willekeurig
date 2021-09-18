pub mod block;
pub mod identifier;
pub mod registry;
pub mod settings;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        fn floor_mod(a: f32, b: f32) -> f32 {
            (a % b + b) % b
        }

        print!("| mod & +divisor | mod & -divisor |\n");
        println!("| :------------- | -------------- |\n");

        let b = 3;
        
        for a in -5..6 {
            print!("|  {} mod {}  =  {} ", a,  b, floor_mod(a as f32,  b as f32));
            println!("|  {} mod {}  =  {} |", a, -b, floor_mod(a as f32, -b as f32));
        }
    }
}