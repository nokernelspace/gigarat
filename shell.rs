use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    // 1. Spawn bash with piped standard input
    // We inherit stdout/stderr so bash prints directly to our terminal screen
    let mut child = Command::new("bash")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start bash process");

    // Take ownership of the child's stdin stream
    let mut bash_stdin = child.stdin.take().expect("Failed to open bash stdin");

    // 2. Enable raw mode (automatically changes ICANON, ECHO, etc.)
    enable_raw_mode()?;

    // Ensure we clean up and restore the terminal if the program panics
    let original_panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        original_panic_hook(panic_info);
    }));

    println!("Shell Active. Piped to bash. Press Ctrl-D to exit.\r");
    print!("$ ");
    io::stdout().flush()?;

    loop {
        // 3. Read structural terminal events instantly
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            
            // Handle Ctrl-D (Exit signal)
            if code == KeyCode::Char('d') && modifiers.contains(KeyModifiers::CONTROL) {
                break;
            }

            match code {
                KeyCode::Enter => {
                    println!("\r");
                    io::stdout().flush()?;

                    // Forward newline to bash and flush the pipe instantly
                    writeln!(bash_stdin)?;
                    bash_stdin.flush()?;

                    print!("$ ");
                    io::stdout().flush()?;
                }
                KeyCode::Char(c) => {
                    // Echo locally for the user to see
                    print!("{}", c);
                    io::stdout().flush()?;

                    // Stream the raw character directly down the pipe
                    write!(bash_stdin, "{}", c)?;
                }
                KeyCode::Backspace => {
                    // Visual terminal erasure locally
                    print!("\x08 \x08");
                    io::stdout().flush()?;

                    // Send a literal backspace code down to bash so its buffer matches ours
                    write!(bash_stdin, "\x08")?;
                    bash_stdin.flush()?;
                }
                _ => {
                    // Ignore other keypresses like function keys for now
                }
            }
        }
    }

    // 4. Clean exit: restore terminal behavior
    println!("\r\nExiting cleanly.");
    disable_raw_mode()?;
    
    // Explicitly drop stdin to signal EOF to bash, letting it shut down gracefully
    drop(bash_stdin);
    let _ = child.wait();

    Ok(())
}

