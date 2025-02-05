use alloy_sol_types::SolType;
use clap::Parser;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::fs::File;
use std::time::Instant;
use tendermint_light_client_verifier::types::LightBlock;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const TENDERMINT_ELF: &[u8] = include_elf!("tendermint-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    // ../files/block_2279100.json
    dir1: String,

    #[clap(long)]
    dir2: String,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.dir1.is_empty() {
        eprintln!("Error: You must specify --dir1");
        std::process::exit(1);
    }

    if args.dir2.is_empty() {
        eprintln!("Error: You must specify --dir2");
        std::process::exit(1);
    }

    let block_file = File::open(args.dir1.clone());
    if block_file.is_err() {
        eprintln!(
            "Error: file {} can not be read {:#?}",
            args.dir1,
            block_file.err()
        );
        std::process::exit(1);
    }

    let block_read: Result<LightBlock, serde_json::Error> =
        serde_json::from_reader(block_file.unwrap());

    if block_read.is_err() {
        eprintln!("Error: light block 1 parse error {:#?}", block_read.err());

        std::process::exit(1);
    }

    let block_1 = block_read.unwrap();

    let block_file = File::open(args.dir2.clone());
    if block_file.is_err() {
        eprintln!(
            "Error: file {} can not be read {:#?}",
            args.dir2,
            block_file.err()
        );
        std::process::exit(1);
    }

    let block_read: Result<LightBlock, serde_json::Error> =
        serde_json::from_reader(block_file.unwrap());

    if block_read.is_err() {
        eprintln!("Error: light block 2 parse error {:#?}", block_read.err());

        std::process::exit(1);
    }

    let block_2 = block_read.unwrap();

    println!("block 1 height {}", block_1.height());
    println!("block 2 height {}", block_2.height());

    // Setup the prover client.
    let client: sp1_sdk::EnvProver = ProverClient::from_env();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    let block_1_encode = serde_cbor::ser::to_vec(&block_1);
    if block_1_encode.is_err() {
        eprintln!("Error: encode block 1 {:#?}", block_1_encode.err());

        std::process::exit(1);
    }

    let block_2_encode = serde_cbor::ser::to_vec(&block_2);
    if block_2_encode.is_err() {
        eprintln!("Error: encode block 2 {:#?}", block_2_encode.err());

        std::process::exit(1);
    }
    // let start = Instant::now();

    stdin.write_vec(block_1_encode.unwrap());
    stdin.write_vec(block_2_encode.unwrap());

    // let (output, report) = client.execute(TENDERMINT_ELF, &stdin).run().unwrap();
    // println!("Program executed successfully.");

    // let duration = start.elapsed();

    // println!("Time elapsed in expensive_function() is: {:?}", duration);

    // let decoded = TendermintOutput::abi_decode(output.as_slice(), true).unwrap();

    // println!("trustedHeight: {:#?}", decoded.trustedHeight);
    // println!("targetHeight: {:#?}", decoded.targetHeight);
    // println!("trustedHeaderHash: {:#?}", decoded.trustedHeaderHash);
    // println!("targetHeaderHash: {:#?}", decoded.targetHeaderHash);

    let (pk, vk) = client.setup(TENDERMINT_ELF);

    // Generate the proof
    let proof = client
        .prove(&pk, &stdin)
        .run()
        .expect("failed to generate proof");

    println!("proof: {}", proof.bytes().len());

    // println!("Number of cycles: {}", report.total_instruction_count());
}

use alloy_sol_types::sol;

sol! {
    /// The public values encoded as a struct that can be easily deserialized inside Solidity.
    struct TendermintOutput {
        uint64 trustedHeight;
        uint64 targetHeight;
        bytes32 trustedHeaderHash;
        bytes32 targetHeaderHash;
    }
}
