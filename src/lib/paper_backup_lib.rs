pub mod lib {

use colored::Colorize;
use std::{
        process::{
            Command,
            Stdio
        },
        io::{ Write, self, BufRead },
        fs::File,
        fs::OpenOptions,
        path::Path,
        thread::sleep,
        time::Duration,
    };
use progress_bar::*;
use qr2term::*;
// use qrcode_generator::{ QrCodeEcc, QrSegment };
use qrcode_png::{QrCode, QrCodeEcc, Color as ColorQr};
use chrono::prelude::*;

    pub enum Menu {
        Help,
        Eff,
        Diceware(String),
        DicewareLock(String),
        Notenum(String),
        Unlock,
    }


    pub fn get_help() {
        println!("\nrequire: ");
        println!("       - rust-diceware binary from crate.io manually installed");
        println!("");
        println!("usage: paper_backup [--help] [--eff]");
        println!("");
        println!("option: ");
        println!("       --help           :  Help command!");
        println!("       --eff            :  Generate Eff random wordlist");
        println!("       --diceware       :  Generate passphrase using diceware crate");
        println!("       --diceware-lock  :  Generate paper backup with --diceware\n");
    }

    pub fn menu_option(menu_list: Menu) {
        match menu_list {
            Menu::Help =>  get_help(),
            Menu::Diceware(arg) => {
                let passphrase = diceware_generate(arg.as_str(),"minilock","-");

                println!("{}", "> diceware".bright_cyan());
                println!("{}", "---------".bright_cyan());
                println!("{}{}", "entropy   : ".cyan(), diceware_generate(arg.as_str(),"minilock","-")[1]);
                println!("{}{}\n", "passphrase: ".green(), passphrase[0]
                         .color("white")
                         .on_color("black")
                         .italic()
                );

            },
            Menu::DicewareLock(arg) => {
                let passphrase = diceware_generate(arg.as_str(),"minilock","-");
                let passphrase_copy = passphrase.clone();

                println!("{}", "> diceware".bright_cyan());
                println!("{}", "---------".bright_cyan());
                println!("{}{}", "entropy   : ".cyan(), diceware_generate(arg.as_str(),"minilock","-")[1]);
                println!("{}{}\n", "passphrase: ".green(), passphrase[0]
                         .color("white")
                         .on_color("black")
                         .italic()
                );

                store_tofile(passphrase_copy[0].to_string());

                println!("{}", gpg_encrypt().unwrap().bright_green());

                let val_for_generate = get_secret_gpg("secret.gpg"); 

                let hash = hashlib_python();

                println!("{}{}", "> Hash thing: ".bright_red(), hash[0]);
                qrcode_generate_to_file(val_for_generate.as_str(), hash[1].as_str(), hash[0].as_str());
                
                println!("{}", shred_helper_files(["secret.gpg","frost"].to_vec()).unwrap().bright_green());

            },
            Menu::Unlock => {
                get_list_qrcode();
            },
            Menu::Eff => {
                println!("\neff wordlist");
                println!("------------");
                println!("{}{}\n","Output: ".green(),  generate_eff_word());
            }
            Menu::Notenum(arg) => {
                println!("> {} {}",arg.bright_red(), "Menu Argument not available please check help: --help".bright_yellow());
            }
        }
    }

    pub fn diceware_generate(n_value: &str, wordlist: &str, delimiter: &str) -> Vec<String> {
        let diceware = Command::new("diceware")
            .args(&["-d", delimiter,"-e","-n", n_value,"-l",wordlist])
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to execute diceware");

        let dice = String::from_utf8_lossy(&diceware.stdout);

        let dice_split = dice.split("\n");

        let dice_vec: Vec<&str> = dice_split.collect();

        dice_vec.into_iter().filter(|v| v.to_string() != "").map(|x| x.to_string()).collect()
    }

    // generate from eff-wordlist crates
    pub fn generate_eff_word() -> String {
        let mut words: Vec<String> = Vec::new();

        let mut words_string = String::new();

        while words.len() < 20 {
            let word = eff_wordlist::large::random_word();
            words.push(word.to_string());
            words_string.push_str(&word.to_string());
            words_string.push('-');
        }

        words_string.pop();


        words_string
    }
    
    pub fn catch_stdin() -> String {
        let mut input = String::new();

        let _ = io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).expect("Something wrong with stdin().read_line() !");

        input.trim().to_string()
    }


    pub fn gpg_encrypt() -> Result<String, String> { 
        let gpg = Command::new("gpg")
            .args(&[
                  "-o","secret.gpg","--symmetric","--s2k-mode","3","--s2k-count","65011712","--s2k-digest-algo",
                  "SHA512","--cipher-algo","AES256","--armor", "frost"
            ])
            .stdout(Stdio::piped())
            .output()
            .expect("> gpg_encrypt() failed!");

        let gpg_utf8 = String::from_utf8_lossy(&gpg.stdout);
        let gpg_utf8_err = String::from_utf8_lossy(&gpg.stderr);

        if gpg_utf8.is_empty() {
            Ok(format!("> gpg_encrypt successfully."))
        } else {
            Err(format!("> something wrong with gpg_utf8_err! : {}", gpg_utf8_err))
        }
    }

    pub fn store_tofile(store_val: String) {

        let store_val_copy = store_val.clone();

        validate_passphrase(store_val_copy);

        let path = Path::new("frost");
        let show_path = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("> couldn't create path {}: {}", show_path, why),
            Ok(file) => file,
        };

        match file.write_all(format!("{}\n", store_val).as_bytes()) {
            Err(why) => panic!("> couldn't write to {}: {}", show_path, why),
            Ok(_) => println!("{}{}", "> successfully wrote to ".purple(), show_path),
        };
    }

    pub fn validate_passphrase(val: String) -> String {
        loop {
            // some kind a time sleep
            print!("{}", "> please validate passphrase : ".bright_blue());
            let check = catch_stdin();

            if check == val {
                println!("{}", "> validate successfully.".bright_green());
                break;
            }

            if check == "--show" {
                println!("{}{}", "> what store passphrase: ".bright_green(), val.yellow());
                // some kind a time sleep
            }
        }
        val.to_string()
    }
    
    pub fn shred_helper_files(val: Vec<&str>) -> Result<String, String> { 

        let mut shred_args:Vec<&str> = Vec::new();
        shred_args.push("-vuzn");
        shred_args.push("20");
        
        let mut what_file_shreding = String::new();
        for v in val {
            what_file_shreding.push_str(v);
            what_file_shreding.push_str(", ");
            shred_args.push(v);
        }
        
        println!("{}{} 20: ", "> Shreding ".magenta(), what_file_shreding.magenta());
        
        let shred = Command::new("shred")
            .args(&shred_args)
            .stdout(Stdio::piped())
            .output()
            .expect("> shred_helper_files failed!");

        let shred_utf8 = String::from_utf8_lossy(&shred.stdout);
        let shred_utf8_err = String::from_utf8_lossy(&shred.stderr);
        
        let shred_copy_split = shred_utf8_err.split("\n");
        let shred_copy_collect: Vec<&str> = shred_copy_split.collect();
        process_bar(shred_copy_collect.len());


        if shred_utf8.is_empty() {
            Ok(format!("{}", "> shred successfully. ".magenta()))
        } else {
            Err(format!("> something wrong with shreding files"))
        }
    }

    pub fn process_bar(val: usize) {
       init_progress_bar(val);
       set_progress_bar_action("*shreding", Color::Magenta, Style::Normal);

       let mut i = 0;
       while i < val {
           sleep(Duration::from_millis(25));
           inc_progress_bar();
           i += 1;
       }
       finalize_progress_bar();
    }

    // Need better generate qrcode
    pub fn qrcode_generate_to_file(val: &str, val2: &str, val3: &str) {

        let utc: DateTime<Utc> = Utc::now();
        let utc_to_png = utc.format("%m%d%y_%H%M").to_string();
        print!("{}", "> Name your qrcode file: ".bright_yellow());
        
        let name_png = catch_stdin();
        let name_png_print = format!("qrcode/{}_{}_{}.png", val2, utc_to_png, name_png);
        let mut qrcode = QrCode::new(val.as_bytes(), QrCodeEcc::Medium).unwrap();

        qrcode.margin(50);
        qrcode.zoom(10);
        
        let buffer = qrcode.generate(ColorQr::Grayscale(0, 255)).unwrap();
        std::fs::write(name_png_print, buffer)
            .expect(format!("{}", ">Something wrong with qrcode_generate write file".red()).as_str());

        print_qr(val).unwrap();

        let status_short = qrcode_with_short_hash(
            val2, 
            utc_to_png.as_str(), 
            name_png.as_str(), 
            val3).unwrap();
        
        println!("{}", status_short);

    }

    fn qrcode_with_short_hash(hash: &str, utc_time: &str, name_png: &str, short_hash: &str) -> Result<String, String> {
        
        let qrcode_short = Command::new("convert")
            .args(&[
                  format!("qrcode/{}_{}_{}.png", hash, utc_time, name_png).as_str(), 
                  "-gravity", "center", "-scale", "200%",
                  "-extent","100%", "-scale", "100%",
                  "-gravity", "south",
                  "-font", "/usr/share/fonts/truetype/noto/NotoMono-Regular.ttf",
                  "-pointsize","24","-fill","black",
                  "-draw", format!("text 0,50 '{}-{}'", short_hash, name_png).as_str(),
                  format!("qrcode/{}_{}_{}.png", short_hash, utc_time, name_png).as_str()
            ])
            .stdout(Stdio::piped())
            .output()
            .expect("> somthing wrong with hashlib_python!");

        let qrcode_short_utf8 = String::from_utf8_lossy(&qrcode_short.stdout);
        if qrcode_short_utf8.is_empty() {
            Ok(format!("{}", "> qrcode_with_short_hash successfully.".green()))
        } else {
            Err(format!("{}","> somthing wrong with qrcode_with_short_hash!".bright_red()))
        }

    }

    pub fn get_secret_gpg(string_path: &str) -> String {
        let mut bucket_val = String::new();
        if let Ok(lines) = read_a_file(string_path) {
            for line in lines {
                if let Ok(val) = line {
                    bucket_val.push_str(val.as_str());
                    bucket_val.push_str("\n");
                }
            }
        }
        bucket_val
    }

    fn read_a_file<T>(filename: T) -> io::Result<io::Lines<io::BufReader<File>>>
    where T: AsRef<Path>, 
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    fn hashlib_python() -> Vec<String> {
        let hashlib_python = Command::new("python3")
            .args(&[
                  "hash.py"
            ])
            .stdout(Stdio::piped())
            .output()
            .expect("> somthing wrong with hashlib_python!");

        let hashlib_python = String::from_utf8_lossy(&hashlib_python.stdout);

        let hashlib_split = hashlib_python.split("\n");
        let hashlib_vec: Vec<&str> = hashlib_split.collect();
        let hashlib_vec_copy = hashlib_vec.clone();

        println!("{}", "> Hashlib python : ".bright_yellow());
        for line in hashlib_vec {
            if line == ""{
                println!();
            }

            if line == "-----END PGP MESSAGE-----" {
                println!("{}",line.bright_green());
                break;
            }
            println!("{}",line.bright_green());
        }

        vec![hashlib_vec_copy[0].to_string(), hashlib_vec_copy[1].to_string()]

    }

    fn get_list_qrcode() {
        let list_of_qrcode = Command::new("ls")
            .args(&[
                 "-a", "qrcode"
            ])
            .stdout(Stdio::piped())
            .output()
            .expect(format!("{}", "> somthing wrong with list_of_qrcode. (qrcode directory not found)"
                            .bright_red()).as_str());

        let list_of_qrcode_utf8 = String::from_utf8_lossy(&list_of_qrcode.stdout);

        if list_of_qrcode_utf8.is_empty() {
            panic!("{}", "No qrcode directory".bright_red());
        }

        let list_of_qrcode_split = list_of_qrcode_utf8.split("\n");

        let list_of_qrcode_vec: Vec<&str> = list_of_qrcode_split.collect();

        let output: Vec<&str> = list_of_qrcode_vec.clone()
            .into_iter()
            .filter(|&x| x != "..".to_string() && x != ".".to_string() && x != "".to_string())
            .collect::<Vec<&str>>();

        println!();
        println!("{}", "List of orcode".yellow());
        println!("{}", "--------------".yellow());
        let mut index = 0;
        while index < output.len() {
            println!("{}. {}", index , output[index].bright_cyan());
            index += 1;
        }

        print!("{}", "\n> chose file name by index or name: ".bright_green());
        let chose = catch_stdin();

        let out_chose = stdin_check_numeric(chose.as_str());

        let mut path_name = String::new();
        if out_chose {
            let index_path_name = chose.trim().parse::<usize>().unwrap();
            path_name.push_str(output[index_path_name]);
        } else {
            path_name.push_str(format!("{}", chose).as_str());
        }

        scan_qrcode(path_name.as_str());

    }

    fn stdin_check_numeric(val: &str) -> bool {
        let chars: Vec<char> = val.chars().collect();
        let chars_copy = chars.clone();
        let mut numeric = 0;
        for char in chars {
            if char.is_digit(10) {
               numeric += 1;
            }
        }

        if numeric == chars_copy.len() {
            true
        } else {
            false
        }
    }


    fn scan_qrcode(name_of_file: &str) {
        let qrcode_name_location = format!("qrcode/{}", name_of_file);
        let zbar = Command::new("zbarimg")
            .args(&[
                  "--nodisplay", "--nodbus", "--quiet",
                  qrcode_name_location.as_str()
            ])
            .stdout(Stdio::piped())
            .output()
            .expect(format!("{}", "somthing wrong with zbar piped()".bright_red()).as_str());

        let zbar_utf8 = String::from_utf8_lossy(&zbar.stdout);
        let zbar_utf8_replace = zbar_utf8.replace("QR-Code:", "");
        let zbar_utf8_split = zbar_utf8_replace.split("\n");
        let zbar_utf8_vec: Vec<&str> = zbar_utf8_split.collect();
        
        // write to qrcode_decode.gpg
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("qrcode_encode.gpg").unwrap();
        
        for line in zbar_utf8_vec.into_iter() {
            writeln!(&mut file,"{}", line).expect(
                format!("{}","> something wrong with writeln!".bright_red())
                .as_str());
        }

        let gpgterm = get_secret_gpg("qrcode_encode.gpg");
        println!("> {}", gpgterm.bright_yellow());

        println!("{}{}", "> VALUE: ".bright_green(), gpg_decrypt().unwrap().bright_yellow());

        shred_helper_files(["qrcode_encode.gpg"].to_vec()).unwrap().bright_green();
    }
    
    pub fn gpg_decrypt() -> Result<String, String> { 
        let gpg = Command::new("gpg")
            .args(&[
                  "--decrypt", "qrcode_encode.gpg"
            ])
            .stdout(Stdio::piped())
            .output()
            .expect("> gpg_decrypt() failed!");

        let gpg_utf8 = String::from_utf8_lossy(&gpg.stdout);
        let gpg_utf8_err = String::from_utf8_lossy(&gpg.stderr);
        let gpg_utf8_split = gpg_utf8.split("\n");
        let gpg_utf8_vec: Vec<&str> = gpg_utf8_split.collect();

        if !gpg_utf8.is_empty() {
            Ok(format!("{}", gpg_utf8_vec[0]))
        } else {
            Err(format!("> something wrong with gpg_utf8_err! : {}", gpg_utf8_err))
        }
    }
}
