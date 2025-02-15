use anyhow::Result;
use vergen_gitcl::{Emitter, GitclBuilder};

fn main() -> Result<()> {
    let gitcl = GitclBuilder::default().build()?;

    let gitcl_res = Emitter::default()
        .idempotent()
        .fail_on_error()
        .add_instructions(&gitcl)
        .and_then(|emitter| emitter.emit());

    if let Err(e) = gitcl_res {
        eprintln!("error occured while generating instructions: {e:?}");
        Emitter::default().idempotent().fail_on_error().emit()?;
    }

    Ok(())
}
