use console::Term;
use std::io::{stdout, Write};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

pub fn confirm(msg: &str) -> bool {
    let mut term = Term::stdout();
    term.write(&format!("{msg} [y/n]: ").into_bytes()).unwrap();
    let result = match term.read_char().unwrap() {
        'Y' | 'y' => true,
        'N' | 'n' => false,
        _ => {
            term.write("\n".as_bytes()).unwrap();
            confirm(msg)
        }
    };
    term.write("\n".as_bytes()).unwrap();
    result
}

const STATES: [char; 8] = ['⢿', '⣻', '⣽', '⣾', '⣷', '⣯', '⣟', '⡿'];

pub struct LoadingAnimation {
    task: JoinHandle<()>,
}

impl LoadingAnimation {
    pub fn new(msg: &'static str) -> Self {
        Self {
            task: tokio::spawn(Self::animate(msg)),
        }
    }

    pub fn finish(&self, msg: &str) {
        self.task.abort();
        print!("\r{msg}  ");
        stdout().flush().unwrap();
        println!();
    }

    async fn animate(msg: &'static str) {
        let i = &mut 0;
        loop {
            Self::frame(i, msg).await;
        }
    }

    async fn frame(i: &mut usize, msg: &'static str) {
        print!("\r{} {}", STATES[*i], msg);
        stdout().flush().unwrap();
        if *i >= 7 {
            *i = 0
        } else {
            *i += 1
        }
        sleep(Duration::from_millis(150)).await;
    }
}
