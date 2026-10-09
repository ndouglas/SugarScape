mod archive;
mod cli;
mod io;
mod manifest;
mod report;
mod report_types;
#[cfg(test)]
mod tests;
mod validate;
mod wire;
pub use cli::cli;
pub fn route(args: &[String]) -> Option<Result<(), String>> {
    let position = args.iter().position(|a| a == "--behavior-trees")?;
    let mut rest = args.to_vec();
    rest.remove(position);
    Some(cli(&rest))
}
