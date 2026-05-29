use clap::Parser;
use portfoliowebsitebuilder_core::{
    discover_content_bundles, generate_site, resolve_project_root, validate_site,
};
use std::io::{self, BufRead, Write};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "portfoliowebsitebuilder")]
struct Args {
    #[arg(long, help = "validate content bundle without writing output")]
    validate: bool,
    #[arg(long, help = "treat unknown top-level keys and unknown widget props keys as errors")]
    strict: bool,
    #[arg(long, help = "list content bundles under content/ and exit")]
    list_sites: bool,
    #[arg(long, help = "after build, serve the output directory over HTTP on localhost")]
    serve: bool,
    #[arg(long, default_value_t = 8080, help = "port for --serve (default 8080)")]
    port: u16,
    #[arg(
        long,
        help = "content bundle path (relative to project root or absolute); skips interactive prompt"
    )]
    site: Option<String>,
}

fn main() {
    let args = Args::parse();
    let code = run_cli(args);
    std::process::exit(code);
}

fn run_cli(args: Args) -> i32 {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let project_root = match resolve_project_root() {
        Ok(p) => p,
        Err(e) => {
            let _ = writeln!(stderr, "Error: {e}");
            return 1;
        }
    };

    if args.list_sites {
        match discover_content_bundles(&project_root) {
            Ok(bundles) => {
                for rel in bundles {
                    let _ = writeln!(stdout, "{rel}");
                }
                0
            }
            Err(e) => {
                let _ = writeln!(stderr, "Error: {e}");
                1
            }
        }
    } else if args.validate {
        if args.serve {
            let _ = writeln!(stderr, "Error: cannot use --serve with --validate");
            return 2;
        }
        let template_dir = project_root.join("Template");
        let site_input = args.site.as_deref().unwrap_or("");
        match validate_site(
            &project_root,
            site_input,
            &template_dir,
            args.strict,
            &mut stdout,
            &mut stderr,
        ) {
            Ok(()) => 0,
            Err(e) => {
                let _ = writeln!(stderr, "Error: {e}");
                1
            }
        }
    } else {
        let site_input = match args.site {
            Some(s) => s,
            None => match resolve_interactive_site_input(&project_root, &mut stdout, &mut stderr) {
                Ok(s) => s,
                Err(e) => {
                    let _ = writeln!(stderr, "Error: {e}");
                    return 1;
                }
            },
        };

        let target_dir = match generate_site(
            &project_root,
            &site_input,
            args.strict,
            &mut stdout,
            &mut stderr,
        ) {
            Ok(d) => d,
            Err(e) => {
                let _ = writeln!(stderr, "Error: {e}");
                return 1;
            }
        };

        if args.serve {
            if args.port < 1 {
                let _ = writeln!(stderr, "Error: invalid --port {}", args.port);
                return 2;
            }
            match portfoliowebsitebuilder_core::serve::serve_static_dir(
                Path::new(&target_dir),
                args.port,
                &mut stdout,
            ) {
                Ok(()) => 0,
                Err(e) => {
                    let _ = writeln!(stderr, "Error: {e}");
                    1
                }
            }
        } else {
            0
        }
    }
}

fn resolve_interactive_site_input(
    project_root: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<String, portfoliowebsitebuilder_core::CoreError> {
    let bundles = discover_content_bundles(project_root)?;
    let stdin = io::stdin();
    let mut lines = stdin.lock();

    loop {
        write!(
            stdout,
            "Enter content bundle directory path (absolute or relative; empty = default or pick from list; ? = list): "
        )?;
        stdout.flush()?;
        let mut line = String::new();
        lines.read_line(&mut line)?;
        let site_input = line.trim().to_string();

        match site_input.as_str() {
            "?" => {
                print_bundle_list(stdout, &bundles)?;
                continue;
            }
            "" if bundles.len() > 1 => {
                print_bundle_list(stdout, &bundles)?;
                write!(stdout, "Enter number or path: ")?;
                stdout.flush()?;
                let mut choice_line = String::new();
                lines.read_line(&mut choice_line)?;
                let choice = choice_line.trim();
                if let Some(resolved) = resolve_bundle_choice(choice, &bundles) {
                    return Ok(resolved);
                }
                return Ok(choice.to_string());
            }
            _ => return Ok(site_input),
        }
    }
}

fn print_bundle_list(stdout: &mut dyn Write, bundles: &[String]) -> io::Result<()> {
    writeln!(stdout, "Available content bundles:")?;
    for (i, rel) in bundles.iter().enumerate() {
        writeln!(stdout, "  {}. {rel}", i + 1)?;
    }
    Ok(())
}

fn resolve_bundle_choice(choice: &str, bundles: &[String]) -> Option<String> {
    let choice = choice.trim();
    if choice.is_empty() {
        return None;
    }
    if let Ok(n) = choice.parse::<usize>() {
        if n >= 1 && n <= bundles.len() {
            return Some(bundles[n - 1].clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_bundle_choice_numbered() {
        let bundles = vec!["content/kometa".into(), "content/my-studio".into()];
        assert_eq!(
            resolve_bundle_choice("2", &bundles),
            Some("content/my-studio".into())
        );
        assert_eq!(resolve_bundle_choice("9", &bundles), None);
    }
}
