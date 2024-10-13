
use std::io;

#[derive(Debug)]

struct Computer {
    brand: String,
    processor_name: String,
    memory_size: i128,
}

impl Computer {
    fn display_pe_rand(&self) {
        println!(
            "Calculatorul are: {0}, {1}, {2}",
            self.brand, self.processor_name, self.memory_size
        );
    }
    fn display_tot_odata(&self) {
        println!("Calculatorul are: {:?}", self); //nu merge habar nu am de ce
    }
}

// fn introd_computer() -> Computer {
//     Computer {
//         brand: String::from("Lenovo"),
//         processor_name: String::from("Ryzen 7 6800HS"),
//         memory_size: 16000,
//     }
// }
fn main() {

    let computer_array = [
    Computer {
        brand: String::from("Lenovo"),
        processor_name: String::from("AMD Ryzen 7 6800HS"),
        memory_size: 16000,
    },
    Computer {
        brand: String::from("Asus"),
        processor_name: String::from("Intel i7 10750H"),
        memory_size: 8000,
        },
    Computer {
        brand: String::from("Lenovo"),
        processor_name: String::from("Intel core ultra 5 125H"),
        memory_size: 32000,
    },
    Computer {
        brand: String::from("Acer"),
        processor_name: String::from("AMD Ryzen 5 7545HS"),
        memory_size: 12000,
    }];

    //computer_array[1].display_pe_rand();
    let mut input = String::new();
    println!("Scrie 'print_all' pt toate, sau 'print_mem' pt cel cu cea mai multa memorie");
    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read");
        input = input.trim().to_string();
        println!("citit: {input}");
        if input == "print_all" {
            computer_array[0].display_pe_rand();
            computer_array[1].display_pe_rand();
            computer_array[2].display_pe_rand();
            computer_array[3].display_pe_rand();
        }
        else {
            if input == "print_mem" {
                computer_array[2].display_tot_odata();
            }
            else {
                break;
            }
        }

        input = "".to_string();
    }
}
