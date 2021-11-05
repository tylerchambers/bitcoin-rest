use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use bitcoin_hashes::sha256d;
use bitcoincore_rpc::{bitcoin, Auth, Client, RpcApi};
use clap::{App as ClapApp, Arg};
use std::path::PathBuf;
use std::str::FromStr;

async fn best_block_hash(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(data.rpc_conn.get_best_block_hash().unwrap().to_string())
}

async fn get_block_by_hash(
    data: web::Data<AppState>,
    block_hash: web::Path<String>,
) -> impl Responder {
    let hash = sha256d::Hash::from_str(&block_hash.into_inner()).unwrap();
    let block = data
        .rpc_conn
        .get_block(&bitcoin::hash_types::BlockHash::from_hash(hash))
        .unwrap();
    HttpResponse::Ok().json(block)
}

async fn get_blockchain_info(data: web::Data<AppState>) -> impl Responder {
    let info = data.rpc_conn.get_blockchain_info().unwrap();
    HttpResponse::Ok().json(info)
}

async fn get_block_count(data: web::Data<AppState>) -> impl Responder {
    let count = data.rpc_conn.get_block_count().unwrap();
    HttpResponse::Ok().json(count)
}

async fn get_block_filter(
    data: web::Data<AppState>,
    block_hash: web::Path<String>,
) -> impl Responder {
    let block_hash = sha256d::Hash::from_str(&block_hash.into_inner()).unwrap();
    let filter = data
        .rpc_conn
        .get_block_filter(&bitcoin::hash_types::BlockHash::from_hash(block_hash))
        .unwrap();
    HttpResponse::Ok().json(filter)
}

async fn get_block_by_height(data: web::Data<AppState>, height: web::Path<u64>) -> impl Responder {
    let block = data.rpc_conn.get_block_hash(height.into_inner()).unwrap();
    HttpResponse::Ok().json(block)
}

async fn get_block_header(
    data: web::Data<AppState>,
    block_hash: web::Path<String>,
) -> impl Responder {
    let block_hash =
        bitcoincore_rpc::bitcoin::BlockHash::from_str(&block_hash.to_string()).unwrap();
    let block_header = data.rpc_conn.get_block_header(&block_hash).unwrap();
    HttpResponse::Ok().json(block_header)
}

// TODO (tylerchambers): Add getblockstats endpoint

async fn get_chain_tips(data: web::Data<AppState>) -> impl Responder {
    let tips = data.rpc_conn.get_chain_tips().unwrap();
    HttpResponse::Ok().json(tips)
}

// TODO (tylerchambers): Add getchainstats endpoint

async fn get_difficulty(data: web::Data<AppState>) -> impl Responder {
    let difficulty = data.rpc_conn.get_difficulty().unwrap();
    HttpResponse::Ok().json(difficulty)
}

// TODO (tylerchambers): Add getmempoolancestors & getmempooldescendants endpoints

async fn get_mempool_entry(data: web::Data<AppState>, txid: web::Path<String>) -> impl Responder {
    let txid = sha256d::Hash::from_str(&txid.into_inner()).unwrap();
    let entry = data
        .rpc_conn
        .get_mempool_entry(&bitcoin::hash_types::Txid::from_hash(txid))
        .unwrap();
    HttpResponse::Ok().json(entry)
}

// TODO (tylerchambers): Add getmempoolinfo endpoint

async fn get_raw_mempool(data: web::Data<AppState>) -> impl Responder {
    let mempool = data.rpc_conn.get_raw_mempool().unwrap();
    HttpResponse::Ok().json(mempool)
}

async fn get_tx_out(
    data: web::Data<AppState>,
    txid: web::Path<String>,
    vout: web::Path<u32>,
) -> impl Responder {
    let txid = sha256d::Hash::from_str(&txid.into_inner()).unwrap();
    let tx_out = data
        .rpc_conn
        .get_tx_out(
            &bitcoin::hash_types::Txid::from_hash(txid),
            vout.into_inner(),
            None,
        )
        .unwrap();
    HttpResponse::Ok().json(tx_out)
}

async fn get_mempool_tx_out(
    data: web::Data<AppState>,
    txid: web::Path<String>,
    vout: web::Path<u32>,
) -> impl Responder {
    let txid = sha256d::Hash::from_str(&txid.into_inner()).unwrap();
    let tx_out = data
        .rpc_conn
        .get_tx_out(
            &bitcoin::hash_types::Txid::from_hash(txid),
            vout.into_inner(),
            Some(true),
        )
        .unwrap();
    HttpResponse::Ok().json(tx_out)
}

async fn get_tx_out_set_info(data: web::Data<AppState>) -> impl Responder {
    let info = data.rpc_conn.get_tx_out_set_info().unwrap();
    HttpResponse::Ok().json(info)
}

struct AppState {
    rpc_conn: Client,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = ClapApp::new("Bitcoin RPC Rest Gateway")
        .version("0.0.1")
        .author("Tyler Chambers <me@tylerchambers.net>")
        .about("Provides a REST interface for Bitcoin core's JSON-RPC API")
        .arg(
            Arg::with_name("node")
                .short("n")
                .long("node")
                .value_name("URL")
                .help("URL of your node. E.g: \"http://localhost:8332.\"")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("cookie")
                .short("c")
                .long("cookie")
                .value_name("PATH")
                .help(
                    "The .cookie file used to authenticate your requests. E.g: \"~/.bitcoin/.cookie\"",
                )
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    HttpServer::new(move || {
        let node = matches.value_of("node").unwrap();
        let cookie = matches.value_of("cookie").unwrap();
        App::new()
            .data(AppState {
                // TODO (tylerchambers): Make these flags / config file options.
                rpc_conn: Client::new(node, Auth::CookieFile(PathBuf::from(cookie))).unwrap(),
            })
            .service(
                web::scope("/api/v1/blockchain/")
                    .route("/bestblock", web::get().to(best_block_hash))
                    .route("/block/{hash}", web::get().to(get_block_by_hash))
                    .route("/info", web::get().to(get_blockchain_info))
                    .route("/blockcount", web::get().to(get_block_count))
                    .route("/blockfilter/{hash}", web::get().to(get_block_filter))
                    .route("/blockhash/{height}", web::get().to(get_block_by_height))
                    .route("/blockheader/{hash}", web::get().to(get_block_header))
                    .route("/chaintips", web::get().to(get_chain_tips))
                    .route("/difficulty", web::get().to(get_difficulty))
                    .route("/txout/{txid}/{vout}/", web::get().to(get_tx_out)),
            )
            .service(
                web::scope("/api/v1/mempool/")
                    .route("/txout/{txid}/{vout}/", web::get().to(get_mempool_tx_out))
                    .route("/raw", web::get().to(get_raw_mempool))
                    .route("/entry/{txid}", web::get().to(get_mempool_entry))
                    .route("/txoutsetinfo", web::get().to(get_tx_out_set_info)),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
