use chrono::prelude::*;
use load_lpp::load_log_dad141::*;
use load_lpp::ERROR_FLT_PARSE;
use load_lpp::{ERROR_STR_GENERAL, ERROR_STR_INVALID, ERROR_STR_NONE, ERROR_STR_SKIPPED};
use std::convert::TryInto;
use std::io::prelude::*;
use std::io::Error;
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::time::Duration;

fn main() {
    let timeout: Duration = Duration::new(15, 0); // seconds, nanoseconds
    let connection_retry: Duration = Duration::new(30, 0); // seconds, nanoseconds
    let write_read_pause: Duration = Duration::new(2, 0); // seconds, nanoseconds

    // get CLI arguments
    let (csv_name, ip, port, mut tcmd_str, minutes, delay, verbose) = parse_cli_log();

    if verbose {
        println!("csv_name {}", csv_name);
        println!("ip {}", ip);
        println!("port {}", port);
        println!("tcmd_str {}", tcmd_str);
        println!("minutes {}", minutes);
        println!("delay {}", delay);
    }

    // Init connection with a closure, which can later be used to refresh the connection if needed.
    // Closures capture the variables in the environment where they are defined.
    // In this case, it is defined here and it captures: socket and timeout.
    // Then, it can be called anywhere, without keeping socket and timeout in scope.
    // This closure takes no arguments because it only needs these environmental-scope variables,
    // no additional argument (at calling time) is required to init the connection.
    // It will be called as closure().
    // Arguments are added |args| when the env variables in the defining env should be combined
    // with arguments given at calling time, i.e., closure(arg1, arg2, ...).
    let ipaddr: Ipv4Addr = ip.parse().expect("arg string is not a valid ip address");
    let socket = SocketAddrV4::new(ipaddr, port);
    let init_connection = || -> Result<TcpStream, Error> {
        let connection = std::net::TcpStream::connect(socket.to_string())?;
        connection.set_nonblocking(false)?;
        connection.set_read_timeout(Some(timeout))?;
        connection.set_write_timeout(Some(timeout))?;
        connection.set_nodelay(true)?;
        Ok(connection)
    };
    let mut connection = init_connection().expect("could not initiate the connection");
    println!("connected to socket {}", socket.to_string());

    // wait for delay if any
    if delay != 0 as u64 {
        println!("starting with delay of {} minute(s)", delay);
        std::thread::sleep(std::time::Duration::from_secs(delay * 60 as u64));
    }

    // telnet command
    tcmd_str.push('\n');
    let tcmd: [u8; 3] = tcmd_str
        .as_bytes()
        .try_into()
        .expect("bug on the telnet the command");

    let mut csvfile = prepare_csvfile(&csv_name);

    // datetime
    let minutes_duration: chrono::Duration = chrono::Duration::minutes(minutes as i64);
    let dt_now: DateTime<Local> = Local::now();
    let mut dtr: DateTime<Local> = chrono_first_rounded(dt_now, minutes_duration);
    let mut dtr_str = dtr.to_rfc3339_opts(SecondsFormat::Secs, false);
    let mut dtr_next = dtr + minutes_duration;
    let mut dtr_next_str = dtr_next.to_rfc3339_opts(SecondsFormat::Secs, false);
    println!(
        "starting at: {}, and then repeating from {} every {} minute(s)",
        dtr_str, dtr_next_str, minutes
    );

    // wait for the starting time
    let mut wait = dtr - Local::now();
    let mut sleep_duration = wait
        .to_std()
        .expect("error in sleeping duration, negative sleep duration?");
    std::thread::sleep(sleep_duration);
    println!("OK, logging ...");

    // init mut variables for tcp logging
    let mut connection_ok = true;
    let mut buffer = [0; 32];
    let mut raw_reading: &str;
    let mut w: f64;

    loop {
        match connection.read(&mut buffer) {
            Ok(b) if b > 0 => println!("warning, found non-empty queue with length: {}", b),
            _ => {}
        }

        match connection.write(&tcmd) {
            Ok(b) if b == 3 => {}
            _ => println!("warning, failed to write command"),
        }

        // a short delay before reading the logger response
        std::thread::sleep(write_read_pause);

        raw_reading = match connection.read(&mut buffer) {
            Ok(0) => {
                println!("{} no data", dtr_str);
                connection_ok = false;
                ERROR_STR_NONE
            }
            Ok(u) => match std::str::from_utf8(&buffer[0..u]) {
                Ok(s) => s.trim_end(),
                Err(e) => {
                    println!("{} IO error, {}", dtr_str, e);
                    connection_ok = false;
                    ERROR_STR_INVALID
                }
            },
            Err(e) => {
                println!("{} IO error, {}", dtr_str, e);
                connection_ok = false;
                ERROR_STR_GENERAL
            }
        };

        w = raw_reading
            .get(2..)
            .map(|s| s.parse().ok())
            .flatten()
            .unwrap_or(ERROR_FLT_PARSE);

        match write!(&mut csvfile, "{},{},{}\n", dtr_str, w, raw_reading) {
            Ok(_) => {
                if verbose {
                    println!(
                        "{}, wrote load {} to {}, raw reading {}; next at {}",
                        dtr_str, w, csv_name, raw_reading, dtr_next_str
                    );
                }
            }
            Err(e) => println!(
                "{}, could not write load {} to file {}, raw reading {}; next at {}",
                dtr_str, w, csv_name, raw_reading, e
            ),
        }

        // recover connection
        while connection_ok == false {
            println!("trying to refresh the connection");
            match init_connection() {
                Ok(c) => {
                    println!("connection successful, resume logging");
                    connection = c;
                    connection_ok = true;
                }
                Err(e) => {
                    println!("connection failed, error {}, trying again ...", e);
                    std::thread::sleep(connection_retry);
                }
            }
        }

        // recover datetime
        while dtr_next <= Local::now() {
            println!(
                "skipping next reading at {} because it has already passed",
                dtr_next_str
            );
            match write!(&mut csvfile, "{},{}\n", dtr_next_str, ERROR_STR_SKIPPED) {
                Ok(_) => {
                    println!(
                        "datetime {}, wrote skipped value {} to file {}",
                        dtr_next_str, ERROR_STR_SKIPPED, csv_name,
                    );
                }
                Err(e) => {
                    println!(
                        "datetime {}, could not write skipped value {} to file {}, error {}",
                        dtr_next_str, ERROR_STR_SKIPPED, csv_name, e
                    );
                }
            }
            dtr_next = dtr_next + minutes_duration;
            dtr_next_str = dtr_next.to_rfc3339_opts(SecondsFormat::Secs, false);
        }

        // wait for the next loop
        wait = dtr_next - Local::now();
        sleep_duration = wait.to_std().unwrap_or(Duration::from_secs(0));
        std::thread::sleep(sleep_duration);
        // prepare for next loop
        dtr = dtr_next;
        dtr_next = dtr + minutes_duration;
        dtr_str = dtr.to_rfc3339_opts(SecondsFormat::Secs, false);
        dtr_next_str = dtr_next.to_rfc3339_opts(SecondsFormat::Secs, false);
    }
}
