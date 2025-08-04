pub fn pause_cli() {
    #[cfg(windows)]
    {
        use std::process::Command;
        let _ = Command::new("cmd").args(["/C", "pause"]).status();
    }

    #[cfg(not(windows))]
    {
        use std::io::{self, Write};
        println!("press any key to exit...");
        io::stdout().flush().unwrap();
        let mut dummy = String::new();
        io::stdin().read_line(&mut dummy).unwrap();
    }
}
