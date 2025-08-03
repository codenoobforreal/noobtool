use std::process::Command;

pub fn pause_cli() {
    #[cfg(windows)]
    {
        let _ = Command::new("cmd").args(["/C", "pause"]).status();
    }

    #[cfg(not(windows))]
    {
        use io::{self, Write};
        println!("按任意键退出...");
        io::stdout().flush().unwrap();
        let mut dummy = String::new();
        io::stdin().read_line(&mut dummy).unwrap();
    }
}
