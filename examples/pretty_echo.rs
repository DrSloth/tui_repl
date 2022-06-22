use tui_repl::Repl;

fn main() -> std::io::Result<()> {
    let mut repl = Repl::new();
    repl.text_mut().push('>');

    repl.run_fullscreen(|cmd: String, out: &mut String| {
        out.push_str("\n>>");
        out.push_str(&cmd);
        out.push_str("\n>");
        Ok(())
    })
}
