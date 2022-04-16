use anyhow::Ok;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use mime::Mime;
use reqwest::{header, Client, Response};
use std::{collections::HashMap, str::FromStr};

#[derive(Parser, Debug)]
#[clap(version = "0.1.0", author = "cicec")]
struct Opts {
    #[clap(subcommand)]
    method: Method,
}

#[derive(Debug, Subcommand)]
enum Method {
    Get(Get),
    Post(Post),
}

#[derive(Debug, Args)]
struct Get {
    #[clap(parse(try_from_str=parse_url))]
    url: String,
}

#[derive(Debug, Args)]
struct Post {
    #[clap(parse(try_from_str=parse_url))]
    url: String,
    #[clap(parse(try_from_str=parse_kv_pair))]
    body: Vec<KvPair>,
}

#[derive(Debug)]
struct KvPair {
    key: String,
    value: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('=');
        let err = || anyhow::anyhow!("invalid key-value pair: {}", s);

        Ok(Self {
            key: split.next().ok_or_else(err)?.to_string(),
            value: split.next().ok_or_else(err)?.to_string(),
        })
    }
}

fn parse_url(s: &str) -> Result<String, anyhow::Error> {
    let url = s.parse()?;

    Ok(url)
}

fn parse_kv_pair(s: &str) -> Result<KvPair, anyhow::Error> {
    KvPair::from_str(s)
}

async fn get(client: Client, args: &Get) -> Result<(), anyhow::Error> {
    let resp = client.get(&args.url).send().await?;

    print_resp(resp).await?;

    Ok(())
}

async fn post(client: Client, args: &Post) -> Result<(), anyhow::Error> {
    let mut body: HashMap<&String, &String> = HashMap::new();

    for pair in args.body.iter() {
        body.insert(&pair.key, &pair.value);
    }

    let resp = client.post(&args.url).form(&body).send().await?;

    print_resp(resp).await?;

    Ok(())
}

async fn print_resp(resp: Response) -> Result<(), anyhow::Error> {
    print_status(&resp);
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

fn print_status(resp: &Response) {
    let status = format!("{:?} {}", &resp.version(), &resp.status()).blue();

    println!("{}\n", status);
}

fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }

    println!("\n");
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        }
        _ => println!("{}", body),
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opts::parse();

    let client = Client::new();

    let result = match opts.method {
        Method::Get(ref args) => get(client, args).await?,
        Method::Post(ref args) => post(client, args).await?,
    };

    Ok(result)
}
