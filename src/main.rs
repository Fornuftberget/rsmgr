use crossbeam_channel::bounded;
use std::process::exit;
use std::thread::{self, JoinHandle};
use structopt::StructOpt;

const DIRECTIVE_PREFIX: &str = "#git_clone ";

fn main() {
    let cfg = rsmgr::cli::Config::from_args();

    if cfg.verbose {
        println!("________________________  ______  ________  ________ ______ ");
        println!("|  ___| ___ \\ ___ \\ ___ \\ | ___ \\/  ___|  \\/  |  __ \\| ___ \\");
        println!("| |_  | |_/ / |_/ / |_/ / | |_/ /\\ `--.| .  . | |  \\/| |_/ /");
        println!("|  _| | ___ \\    /|  __/  |    /  `--. \\ |\\/| | | __ |    / ");
        println!("| |   | |_/ / |\\ \\| |     | |\\ \\ /\\__/ / |  | | |_\\ \\| |\\ \\ ");
        println!("\\_|   \\____/\\_| \\_\\_|     \\_| \\_|\\____/\\_|  |_/\\____/\\_| \\_|");
        println!("                                                            ");
        println!("                                                            ");
    }

    if !cfg.base_path.exists() {
        match std::fs::create_dir(&cfg.base_path) {
            Ok(_) => println!("Creating base directory {}", &cfg.base_path.display()),
            Err(e) => {
                println!("Unable to create resource base path: {}", e);
                exit(1)
            }
        };
    }

    if !cfg.config.exists() {
        println!("Config '{}' does not exist", cfg.config.display());
        exit(1)
    }

    let nthreads = match cfg.num_workers {
        Some(v) => v,
        None => num_cpus::get(),
    };

    let (sender, receiver) = bounded::<String>(nthreads * 4);

    let workers: Vec<JoinHandle<()>> = (0..nthreads)
        .map(|_| {
            let receiver = receiver.clone();
            let params = cfg.clone();

            thread::spawn(move || {
                for res in receiver.iter() {
                    match rsmgr::resource::update(&res, &params) {
                        Ok(msg) => {
                            if !msg.is_empty() {
                                println!("{}", msg)
                            }
                        }
                        Err(e) => eprintln!("Error when updating {}: {}", res, e),
                    }
                    if receiver.is_empty() {
                        break;
                    }
                }
            })
        })
        .collect();

    let mut x = 0;
    std::fs::read_to_string::<_>(&cfg.config)
        .expect("Unable to read resource config file")
        .lines()
        .filter(|line| line.starts_with(DIRECTIVE_PREFIX) && line.contains(":"))
        .for_each(|line| {
            x += 1;
            match line.strip_prefix(DIRECTIVE_PREFIX) {
                Some(res) => sender.send(res.to_string()).unwrap(),
                None => {}
            }
        });

    if cfg.verbose {
        println!("Found {} resources", x);
    }

    drop(sender);

    for w in workers.into_iter() {
        w.join().unwrap();
    }
}
