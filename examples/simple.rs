use tui_repl::Repl;

fn main() -> std::io::Result<()> {
    let mut repl = Repl::new();
    repl.text_mut().push('>');

    repl.run_fullscreen(|cmd: String, out: &mut String| {
        let parts = cmd.split(' ').filter(|s| !s.is_empty()).collect::<Vec<_>>();
        match parts.get(0).map(|s| *s) {
            Some("echo") => {
                out.push_str("\n>>");
                if let Some(s) = parts.get(1) {
                    out.push_str(&s);
                }
                out.push('\n');
            }
            Some("clear") => out.truncate(0),
            _ => out.push('\n'),
        }
        out.push('>');
        Ok(())
    })
}
