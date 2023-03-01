use std::io;
use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use clap::{Arg, App};
use std::time::{Instant};
use std::fs::OpenOptions;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};

/// Data structure for a raw_line of text data
struct Data {
    raw_line: String,
    is_juicy: bool,
    content_type: &'static str,
    exact: String,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            raw_line: "0".to_string(),
            content_type: "None",
            is_juicy: false,
            exact: "None".to_string(),
        }
    }
}

impl Data {
    fn new(raw_line: String) -> Self {
        Self { 
            raw_line,
            ..Default::default() 
        }
    }

    fn to_message(&self) -> String {
        /*
        Forms a messages from the content type and the text
        */
        format!("{}: {}", self.content_type, self.raw_line)

    }

    fn to_exact_message(&self) -> String {
        format!("{}: {}", self.content_type, self.exact)
    }


    // fn to_row(&self) -> String {
    //     /*
    //     Converts the line to a CSV row
    //     */
    //     format!("{},{}", self.content_type, self.raw_line)
    // }

    fn set_content_type(&mut self, regex_map: &HashMap<&'static str, Regex>) -> () {
        /* Searches through the specified regexes to determine if the data 
        provided is valuable information for the provided user
        :param regex_map: Created regexes to search through
        */
        for (content_type, regex) in regex_map.iter() {
            if let Some(capture) = regex.captures(&self.raw_line) {
                if let Some(file_name) = capture.get(1) {
                    // Select first capture group and strip all whitespaces.
                    self.exact = file_name.as_str().to_owned().chars().filter(|c| !c.is_whitespace()).collect::<String>();
                    self.content_type = content_type;
                    self.is_juicy = true;
                    break;
                }
            }
        }
    }
}



struct DataSurgeon {
    matches: clap::ArgMatches,
    output_file: String,
    filename: String,
    clean: bool,
    is_output: bool,
}


impl Default for DataSurgeon {
    fn default() -> Self {
        Self {
            matches: App::new("DataSurgeon: https://github.com/Drew-Alleman/DataSurgeon")
        .version("1.0")
        .author("Drew Alleman")
        .about("DataSurgeon (ds) extracts sensitive information from standard input for incident response, penetration testing, and CTF challenges, including emails, credit cards, URLs, IPs, MAC addresses, and SRV DNS records. ")
        .arg(Arg::with_name("file")
            .short('f')
            .long("file")
            .help("File to extract information from")
            .takes_value(true)
        )
        .arg(Arg::with_name("clean")
            .short('C')
            .long("clean")
            .help("Attempt to remove some of the clean information that might have been sent back")
            .takes_value(false)
        )
        .arg(Arg::with_name("output")
            .short('o')
            .long("output")
            .help("Output's the results of the procedure to a local file (recommended for large files)")
            .takes_value(true)
        )
        .arg(Arg::with_name("time")
            .short('t')
            .long("time")
            .help("Time how long the operation took")
            .takes_value(false)
        )
        .arg(Arg::with_name("email")
            .short('e')
            .long("email")
            .help("Used to extract email addresses from the specifed file or output stream")
            .takes_value(false)
        )
        .arg(Arg::with_name("hashes")
            .short('H')
            .long("hashes")
            .help("Used to extract supported hashes (NTLM, LM, bcrypt, Oracle, MD5, SHA-1, SHA-224, SHA-256, SHA-384, SHA-512, SHA3-224, SHA3-256, SHA3-384, SHA3-512, MD4) from the specified file or output stream")
            .takes_value(false)            
        )
        .arg(Arg::with_name("ip_address")
            .short('i')
            .long("ip-addr")
            .help("Extracts IP addresses from the desired file")
            .takes_value(false)
        )
        .arg(Arg::with_name("ipv6_address")
            .short('6')
            .long("ipv6-addr")
            .help("Extracts IPv6 addresses from the desired file")
            .takes_value(false)
        )
        .arg(Arg::with_name("mac_address")
            .short('m')
            .long("mac-addr")
            .help("Extract's MAC addresses")
            .takes_value(false)
        )
        .arg(Arg::with_name("credit_card")
            .short('c')
            .long("credit-card")
            .help("Extract credit card numbers")
            .takes_value(false)
        )
        .arg(Arg::with_name("url")
            .short('u')
            .long("url")
            .help("Extract url's")
            .takes_value(false)
        )
        .arg(Arg::with_name("domain_users")
            .short('D')
            .long("domain-users")
            .help("Extract possible Windows domain user accounts")
            .takes_value(false)
        )
        .arg(Arg::with_name("files")
            .short('F')
            .long("files")
            .help("Extract filenames")
            .takes_value(false)
        )
        .arg(Arg::with_name("srv_dns")
            .short('d')
            .long("dns")
            .help("Extract Domain Name System records")
            .takes_value(false)
        )
        .get_matches(),
            output_file: "".to_string(),
            filename: "".to_string(),
            clean: false,
            is_output: false,
        }
    }
}


impl  DataSurgeon {

    fn new() -> Self {
        Self { 
            ..Default::default() 
        }
    }

    fn build_regex_query(&self) -> HashMap<&'static str, Regex>{
        /* Builds a regex query to search for important information 
        :return: A HashMap containg the content type and the regex associated 

        Hello, Contributers! 
        To add a new regex, add a new raw_line to the following line. 
        The key is the name of the content you are searching for, 
        and the value is the associated regex.

        ALL REGEXES MUST HAVE THE TARGET ITEM IN THE FIRST CAPTURE GROUP (just use chatGPT)

        let regex_map: HashMap<&str, Regex> = [
                ("test_regex", Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap()),
            ].iter().cloned().collect();

        The key is also used to display to the user what was found, so make it clear and concise, e.g., "email_address: Matched content."
        Note that the regex patterns must conform to Rust's regex syntax. You can test your regex patterns at https://regexr.com/.
        */
        let regex_map: HashMap<&str, Regex> = [
            ("credit_card", Regex::new(r"\b(\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4})\b").unwrap()),
            ("email", Regex::new(r"\b([A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,4})\b").unwrap()),
            ("domain_users", Regex::new(r"\b^([a-zA-Z]+\.[a-zA-Z]+|[a-zA-Z]+)[\w-]*(?=@([a-zA-Z0-9]{1,15}\.(local|network|lan|local)))\b").unwrap()),
            ("url", Regex::new(r"((?:https?|ftp)://[^\s/$.?#].[^\s]*)").unwrap()),
            ("ip_address", Regex::new(r"\b((?:\d{1,3}\.){3}\d{1,3})\b").unwrap()),
            ("ipv6_address", Regex::new(r"([0-9a-fA-F]{1,4}(:[0-9a-fA-F]{1,4}){7})").unwrap()),
            ("srv_dns", Regex::new(r"\b(.+?)\s+IN\s+SRV\s+\d+\s+\d+\s+\d+\s+(.+)\b").unwrap()),
            ("mac_address", Regex::new(r"([0-9a-fA-F]{2}(:[0-9a-fA-F]{2}){5})").unwrap()),
            ("files", Regex::new(r"([\w,\s-]+\.(txt|pdf|doc|docx|xls|xlsx|xml|jpg|jpeg|png|gif|bmp|csv|json|yaml|log|tar|tgz|gz|zip|rar|7z|exe|dll|bat|ps1|sh|py|rb|js|mdb|sql|db|dbf|ini|cfg|conf|bak|old|backup|pgp|gpg|aes|dll|sys|drv|ocx|pcap|tcpdump))").unwrap()),
            ("hashes", Regex::new(r"\b([0-9a-fA-F]{32}|[0-9a-fA-F]{40}|[0-9a-fA-F]{56}|[0-9a-fA-F]{64}|[0-9a-fA-F]{96}|[0-9a-fA-F]{128}|[0-9a-fA-F]{56}|[0-9a-fA-F]{128}|[0-9a-fA-F]{224}|[0-9a-fA-F]{256}|[0-9a-fA-F]{384}|[0-9a-fA-F]{512}|[a-fA-F0-9*]{16}|[a-fA-F0-9*]{40}|[a-fA-F0-9*]{64}|[a-fA-F0-9*]{96}|[a-fA-F0-9*]{128})\b").unwrap())
        ].iter().cloned().collect();
        let keys: Vec<&str> = regex_map.keys().copied().collect();
        /*
        If the user didn't specify any extraction choices (e.g: email, url, ip_address)
        */
        if keys.iter().all(|value_name| !self.matches.is_present(value_name)) {
            return regex_map;
        }
        /*
        If they did, then remove the ones they didnt select
        */
        let filtered_map: HashMap<&str, Regex> = keys
            .into_iter()
            .filter(|&key| {
                let has_match = self.matches.is_present(key); 
                let is_empty = regex_map[key].as_str().is_empty();
                has_match && !is_empty

            })
            .map(|key| (key, regex_map[key].clone()))
            .collect();
        filtered_map
    }

    fn write_to_file(&self, message: String) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.output_file)
            .expect("Failed to open output file");

        writeln!(file, "{}", message).expect("Failed to write to output file");
    }

    fn handle(&self, line: &std::io::Result<String>, regex_map: &HashMap<&'static str, Regex>) {
        /* Handles a line of text and applies various regexes to determine if the 
        content is important
        :param line: Line to process
        :param regex_map: Regexes to apply
        */
        let line = match line {
            Ok(line) => line,
            Err(_) => {
                return;
            }
        };
        if line.is_empty() {
            return;
        }
        let mut data: Data = Data::new(line.to_string());
        data.set_content_type(regex_map);
        if data.is_juicy {
            if self.is_output {
                let mut message: String = data.to_message();
                if self.clean { 
                    message = data.to_exact_message()
                }
                self.write_to_file(message);
                return;
            } 
            if self.clean {
                println!("{}", data.to_exact_message());
            } else {
                println!("{}", data.to_message());
            }
        }
    }

    fn build_arguments(&mut self) {
        /*
        Used to build the attributes in the clap args
        */
        self.output_file =  self.matches.value_of("output").unwrap_or_default().to_string();
        self.is_output =  !self.output_file.is_empty();
        self.clean = self.matches.is_present("junk");
        self.filename = self.matches.value_of("file").unwrap_or("").to_string();
    }


    fn iterate_file(&mut self) {
        /* Iterates through the specified file to find important information
        :param path: file to process
        */
        let file = File::open(Path::new(self.filename.as_str())).unwrap();
        let reader = BufReader::new(file);
        let regex_map = self.build_regex_query();
        for line in reader.lines() {
            self.handle(&line, &regex_map);
        }

    }

    fn iterate_stdin(&mut self) {
        /* Iterates through the standard input to find important informatio
        :param path: file to process
        */
        println!("[*] Reading standard input. If you meant to analyze a file use 'ds -f <FILE>' (ctrl+c to exit)");
        let stdin = io::stdin();
        let reader = stdin.lock();
        let regex_map = self.build_regex_query();
        for line in reader.lines() {
            self.handle(&line, &regex_map);
        }

    }

    fn display_time(&self, elapsed: f32) -> () {
        /* Displays how long the program took
        :param elapsed: Time in f32 that has elapsed.
        */    
        let hours = (elapsed / 3600.0) as u32;
        let minutes = ((elapsed / 60.0) as u32) % 60;
        let seconds = (elapsed as u32) % 60;
        let hours12 = if hours == 0 { 0 } else if hours > 12 { hours - 12 } else { hours };
        println!("Time elapsed: {:02}h:{:02}m:{:02}s", hours12, minutes, seconds);
    }

    fn process(&mut self) {
        /* Searches for important information if the user specified a file othewise 
        the standard output is iterated through
        */    
        let time: bool = self.matches.is_present("time");
        self.build_arguments();
        let start = Instant::now();
        if !self.filename.is_empty() {
            self.iterate_file();
        } else {
            self.iterate_stdin();
        }
        if time {
            self.display_time(start.elapsed().as_secs_f32());
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    /*
    1. Creates the arguments parser
    2. Creates an instance of DataSurgeon
    3. Calls DataSurgeon.process()
    */
    let mut ds = DataSurgeon::new();
    ds.process();
    Ok(())
}
