use spammer::Messanger;
use std::io;

fn main() {
    println!("Do you really want to start ? y/n");
  
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .expect("Filed to read line");
  
    match &answer.trim()[..] {
        "y" => {
            println!("Starting");
            Messanger::build().run().expect("Error");
        },
        _ => println!("Bye"),
    }
}
