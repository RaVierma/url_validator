use std::{
    io::Error,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use csv::{ByteRecord, StringRecord};
use reqwest::{header::USER_AGENT, Client, Url};
use structopt::StructOpt;
use tqdm::tqdm;

#[derive(StructOpt, Debug)]
#[structopt(name = "url_validator")]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "validateurl", about = "Validate the url")]
    ValidateUrl {
        #[structopt(short, long, name = "url", help = "Url you want to validate")]
        url: String,
        #[structopt(short, long, name = "timeout", help = "How long wait for response")]
        timeout: u32,
    },
    #[structopt(name = "validatecsvurl", about = "CSV File having url column")]
    ValidateCSVUrl {
        #[structopt(short, long, name = "csvfile", about = "CSV file path")]
        csvfile: String,
        #[structopt(
            short,
            long,
            name = "urlcolumn",
            help = "comman seperated column name containing url"
        )]
        urlcolumn: String,
        #[structopt(short, long, name = "outputfile", help = "Name of output column")]
        outpufile: String,
        #[structopt(short, long, name = "timeout", help = "How long wait for response")]
        timeout: u32,
    },
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let opt = Opt::from_args();

    match opt.command {
        Command::ValidateUrl { url, timeout } => validate_url(url, timeout)
            .await
            .expect("Unable to validate url"),
        Command::ValidateCSVUrl {
            csvfile,
            urlcolumn,
            outpufile,
            timeout,
        } => validate_csv_url(csvfile, urlcolumn, outpufile, timeout)
            .await
            .expect("Unable to validate csv urls"),
    }

    Ok(())
}

async fn check_url(url: String, timeout: u32) -> Result<String, Error> {
    let url_parse = Url::parse(&url);
    if url_parse.is_err() {
        return Err(Error::new(
            std::io::ErrorKind::Other,
            "Not valid Url".to_string(),
        ));
    }
    let resp = Client::new()
        .get(url.clone())
        .header(
            USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:128.0) Gecko/20100101 Firefox/128.0",
        )
        .timeout(Duration::from_secs(timeout as u64))
        .fetch_mode_no_cors()
        .send()
        .await;

    if resp.is_err() {
        if resp.err().unwrap().is_timeout() {
            return Err(Error::new(std::io::ErrorKind::Other, "timeout".to_string()));
        }
        return Err(Error::new(
            std::io::ErrorKind::Other,
            "not working".to_string(),
        ));
    }
    let resp_status = resp.unwrap().status();

    if !resp_status.is_server_error() {
        Ok(String::from("working"))
    } else {
        Ok(String::from("not working"))
    }
}

async fn validate_url(url: String, timeout: u32) -> Result<(), std::io::Error> {
    println!("*** Validating the url ... ***");
    let url_chk = check_url(url.clone(), timeout).await;
    if url_chk.is_err() {
        println!("=> {} is {}", url, url_chk.err().unwrap().to_string());
    } else {
        println!("=> {} is {}", url, url_chk.unwrap());
    }

    Ok(())
}

async fn proccess_record(record: StringRecord, url_cols: Vec<u8>, timeout: u32) -> ByteRecord {
    let mut new_record = record.clone();
    for col in url_cols.into_iter() {
        let url = record.get(col.into()).unwrap();

        let is_valid = check_url(url.to_string(), timeout).await;
        if is_valid.is_err() {
            new_record.extend(vec![is_valid.err().unwrap().to_string()])
        } else {
            new_record.extend(vec![is_valid.unwrap()]);
        }
    }
    new_record.as_byte_record().clone()
}

async fn validate_csv_url(
    csv_file: String,
    url_column: String,
    out_file: String,
    timeout: u32,
) -> Result<(), std::io::Error> {
    let pth = Path::new(&csv_file);
    if !pth.is_file() {
        panic!("=> File not found... ");
    }

    if !csv_file.contains(".csv") {
        panic!("=> Not a csv file");
    }

    let rdr = csv::Reader::from_path(pth);
    if rdr.is_err() {
        panic!("=> Not a valid csv file...");
    }

    let mut rdr_f = rdr.unwrap();
    let header = rdr_f.headers();

    if header.is_err() {
        panic!("=> Header not found");
    }

    let mut csv_header = header.unwrap().clone();

    let mut url_columns = vec![];
    for (idx, val) in csv_header.into_iter().enumerate() {
        if url_column.contains(val) {
            url_columns.push(idx as u8);
        }
    }

    // adding the status field in header
    for col in url_column.split(",") {
        csv_header.extend(vec![format!("{col}_Status")])
    }

    // shared csv writer using Arc and Mutex
    let new_csv = Arc::new(Mutex::new(csv::Writer::from_path(out_file).unwrap()));

    // storing the joinhandle for furture result extraction i.e after complete
    let mut tasks = vec![];
    for (idx, result) in rdr_f.records().enumerate() {
        let record = result.unwrap();
        let new_csv = new_csv.clone();
        let urlcol = url_columns.clone();
        if idx == 0 {
            let csvheader = csv_header.clone();

            new_csv
                .lock()
                .unwrap()
                .write_byte_record(csvheader.as_byte_record())
                .unwrap();
        }

        let handler = tokio::spawn(async move {
            let is_valid = proccess_record(record, urlcol.to_vec(), timeout).await;

            let _ = new_csv
                .lock()
                .unwrap()
                .write_byte_record(&is_valid)
                .unwrap();
        });
        tasks.push(handler);
    }

    for task in tqdm(tasks).desc(Some("URL Validation: ")) {
        let _ = task.await.unwrap();
    }

    new_csv.lock().unwrap().flush().unwrap();

    Ok(())
}
