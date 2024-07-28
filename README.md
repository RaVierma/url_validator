# URL Validator (In Rust)
This is a URL validator written in Rust in programming language. This also support bulk url validation.

## Features

- **Single or Mulitple Url**: You can validate single url or you can provide comma separated multiple ulrs.
- **CSV**: You can also validate csv having url.
- **timeout**: You also have option to specify the request timeout.

## Installation and build

Before installing the URL Vlaidator, ensure you have the following prerequisites:

1. **Rust**: Make sure you have Rust installed and version must be 1.79 or higher. You can install it via [rustup](https://www.rust-lang.org/tools/install).

Now you're ready to install the  url_validator. Clone this repository and build the application using Cargo:
```bash
    git clone https://github.com/your_username/url_validator.git
    cd url_validator
    cargo build --release
```

## Usage

- Go to release Directory
```bash
    cd url_validator/target/release
```

- Check for avialble subcommands
```bash
    ./url_validator -h
```

- Check the url status of below site with timeout 5 secs.
```bash
    ./url_validator validateurl -u https://www.example.com -t 5
```

- Check the multiple url status with timeout 5 secs.
```bash
    ./url_validator validateurl -u https://www.example.com,https://www.example2.com -t 5
```

- Valid the csv contain urls in column 'website_col1'. Output filename is 'output_filename'.csv and timeout is 5 secs.
```bash
    ./url_validator validatecsvurl -c /path/to/file.csv -o output_filename.csv -u website_col1 -t 5
```
