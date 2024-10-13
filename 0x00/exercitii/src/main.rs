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

fn introd_computer() -> Computer {
    Computer {
        brand: String::from("Lenovo"),
        processor_name: String::from("Ryzen 7 6800HS"),
        memory_size: 16000,
    }
}

fn divizibil(x: i64, n: i64) -> i8 {
    if x % n == 0 {
        return 1;
    } else {
        return 0;
    }
}

fn main() {
    println!("My name is Stefan");

    let a = 10;
    let b = 15;
    if a > b {
        println!("{a}");
    } else {
        println!("{b}");
    }

    let x = 69;
    let n = 3;
    if divizibil(x, n) == 1 {
        println!("E divizibil");
    } else {
        println!("Nu e divizibil");
    }

    let mare_array = [1, 2, 3, 4, 12, 69, 66, 78, 53];
    let mut length_array = mare_array.len();
    let mut max = 0;
    while length_array > 0 {
        length_array = length_array - 1;
        if mare_array[length_array] > max {
            max = mare_array[length_array]
        }
    }

    println!("Maximul din vector este {max}");

    let calculatorul_meu = introd_computer();
    calculatorul_meu.display_pe_rand();
    calculatorul_meu.display_tot_odata();
}
