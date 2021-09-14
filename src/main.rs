mod account;
mod processor;
mod transaction;

use csv::{ReaderBuilder, Trim, Writer};
use processor::TransactionProcessor;
use std::fs::File;
use std::io::ErrorKind;
use transaction::Transaction;

fn main() -> Result<(), std::io::Error> {
    let filename = filename()?;

    let mut transaction_processor = TransactionProcessor::new();
    process_file(&filename, |t| transaction_processor.preprocess(t))?;
    process_file(&filename, |t| transaction_processor.process(t))?;

    write_accounts(transaction_processor)
}

fn process_file(
    filename: &str,
    mut callback: impl FnMut(&Transaction),
) -> Result<(), std::io::Error> {
    let file = File::open(filename)?;
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_reader(file);
    for transaction in reader.deserialize().flatten() {
        callback(&transaction);
    }
    Ok(())
}

fn write_accounts(transaction_processor: TransactionProcessor) -> Result<(), std::io::Error> {
    // For an empty input file, this produces an empty output file (i.e. no headers) - is this OK?
    let mut writer = Writer::from_writer(std::io::stdout());
    for account in transaction_processor.into_accounts().values() {
        writer.serialize(account)?;
    }
    Ok(())
}

fn filename() -> Result<String, std::io::Error> {
    std::env::args() // Not handling non-unicode arguments
        .skip(1) // Ignore the first argument - it's the program name
        .take(1) // Ignore any extra arguments
        .next()
        .ok_or_else(|| {
            std::io::Error::new(
                ErrorKind::InvalidInput,
                "Please provide the CSV input filename",
            )
        })
}
